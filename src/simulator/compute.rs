// Standard library
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

// External libraries
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::sync::{self, GpuFuture, NowFuture};

// CELL
use super::simulator::ComputeOP;
use crate::automaton::{CPUComputableAutomaton, GPUComputableAutomaton, PipelineInfo, Transcoder};
use crate::grid::{Dimensions, Grid, GridHistoryOP};

pub struct GPUCompute<A: GPUComputableAutomaton>
where
    A::Cell: Transcoder,
{
    device: Arc<Device>,
    queue: Arc<Queue>,
    gpu_bufs: Vec<Arc<DeviceLocalBuffer<[u32]>>>,
    nodes: Vec<ComputeNode>,
    next: usize,
    grid_dim: Dimensions,
    _marker: PhantomData<A>,
}

impl<A: GPUComputableAutomaton> GPUCompute<A>
where
    A::Cell: Transcoder,
{
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        nb_nodes: usize,
        initial_grid: &Grid<A::Cell>,
    ) -> Self {
        if nb_nodes < 2 {
            panic!(ERR_NB_NODES)
        }
        let pipe_info = A::vk_setup(&device);
        let pc = A::push_constants(&initial_grid);

        let dim = *initial_grid.dim();
        let total_size = dim.size() as usize;
        let mut gpu_bufs = Vec::with_capacity(nb_nodes);
        for _ in 0..nb_nodes {
            let q_family = vec![queue.family()];
            gpu_bufs.push(
                DeviceLocalBuffer::array(
                    Arc::clone(&device),
                    total_size,
                    BufferUsage::all(),
                    q_family,
                )
                .unwrap(),
            )
        }

        let mut nodes = Vec::with_capacity(nb_nodes);
        for i in 0..nb_nodes {
            let j = {
                if i == nb_nodes - 1 {
                    0
                } else {
                    i + 1
                }
            };
            nodes.push(ComputeNode::new::<A>(
                Arc::clone(&device),
                Arc::clone(&queue),
                &pipe_info,
                Arc::clone(&gpu_bufs[i]),
                Arc::clone(&gpu_bufs[j]),
                pc,
                &dim,
            ))
        }

        // Call reset with the initial grid before returning
        let mut compute = Self {
            device,
            queue,
            gpu_bufs,
            nodes,
            next: 0,
            grid_dim: dim,
            _marker: PhantomData,
        };
        compute.reset(initial_grid);
        compute
    }

    pub fn dispatch(
        mut self,
        rx_op: Receiver<ComputeOP<A>>,
        tx_data: Sender<GridHistoryOP<A::Cell>>,
    ) {
        loop {
            match rx_op.recv() {
                Ok(op) => match op {
                    ComputeOP::Reset(grid) => self.reset(&grid),
                    ComputeOP::Run(nb_gens) => {
                        if !self.run(nb_gens, &tx_data) {
                            break; // A send operation failed, we must terminate ourself
                        }
                    }
                },
                Err(_) => break, // Sender died, time to die
            }
        }
    }

    fn reset(&mut self, initial_grid: &Grid<A::Cell>) {
        // Reset pointer
        self.next = 0;

        // Put data in first GPU buffer
        let cpu_buf = CpuAccessibleBuffer::from_iter(
            Arc::clone(&self.device),
            BufferUsage::transfer_source(),
            false,
            initial_grid.encode().into_iter(),
        )
        .unwrap();
        let cmd = AutoCommandBufferBuilder::primary_one_time_submit(
            Arc::clone(&self.device),
            self.queue.family(),
        )
        .unwrap()
        .copy_buffer(cpu_buf, self.gpu_bufs[0].clone())
        .unwrap()
        .build()
        .unwrap();
        sync::now(Arc::clone(&self.device))
            .then_execute(self.queue.clone(), cmd)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }

    fn run(&mut self, nb_gens: usize, tx_data: &Sender<GridHistoryOP<A::Cell>>) -> bool {
        // Total number of compute nodes
        let nb_nodes = self.nodes.len();

        let wrap_ptr = |ptr| {
            if ptr == nb_nodes - 1 {
                0
            } else {
                ptr + 1
            }
        };

        let min = |a, b| {
            if a < b {
                a
            } else {
                b
            }
        };

        let now_future = |device| Box::new(sync::now(device)) as Box<dyn GpuFuture>;

        let wait_for_future = |future: Box<dyn GpuFuture>| {
            future
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap()
        };

        // Countdown on number of generations that must still be computed
        let mut gens_to_compute = nb_gens;

        // Number of execution futures chained together
        let mut launch_cnt = min(nb_nodes, gens_to_compute);

        // Update next pointer for further calls to run()
        let start_node = self.next;
        self.next = (self.next + gens_to_compute) % nb_nodes;

        // Launch command buffers
        let mut next_exe_node = start_node;
        let mut exe_future = now_future(Arc::clone(&self.device));
        for _i in 0..launch_cnt {
            exe_future = Box::new(self.nodes[next_exe_node].exe(exe_future));
            next_exe_node = wrap_ptr(next_exe_node)
        }
        wait_for_future(exe_future);

        loop {
            // Tell all compute nodes to bring back data to CPU
            let mut next_cpy_node = start_node;
            let mut cpy_futures = VecDeque::with_capacity(launch_cnt);
            for _i in 0..launch_cnt {
                cpy_futures.push_back((self.nodes[next_cpy_node].cpy(), next_cpy_node));
                next_cpy_node = wrap_ptr(next_cpy_node);
            }

            // Update counters and reset exe future
            gens_to_compute -= launch_cnt;
            launch_cnt = min(nb_nodes, gens_to_compute);
            let mut exe_future = now_future(Arc::clone(&self.device));

            // Start reading back data and re-launch computations as needed
            let mut left_to_exe = launch_cnt;
            loop {
                match cpy_futures.pop_front() {
                    Some((future, idx)) => {
                        // Wait for the copy operation to complete
                        future
                            .then_signal_fence_and_flush()
                            .unwrap()
                            .wait(None)
                            .unwrap();

                        // This node is available for compute again
                        if left_to_exe > 0 {
                            exe_future = Box::new(self.nodes[idx].exe(exe_future));
                            left_to_exe -= 1;
                        }

                        // Transform raw data into Grid and send to GridHistory
                        let encoded = Arc::clone(&self.nodes[idx].cpu_out);
                        let grid = Grid::decode(encoded, &self.grid_dim);
                        if let Err(_) = tx_data.send(GridHistoryOP::Push(grid)) {
                            return false;
                        }
                    }
                    None => break, // All copies fetched, leave the inner loop
                }
            }

            if launch_cnt == 0 {
                return true;
            }
            wait_for_future(exe_future);
        }
    }
}

struct ComputeNode {
    device: Arc<Device>,
    queue: Arc<Queue>,
    cpu_out: Arc<CpuAccessibleBuffer<[u32]>>,
    cmd_exe: Arc<AutoCommandBuffer>,
    cmd_cpy: Arc<AutoCommandBuffer>,
}

impl ComputeNode {
    fn new<A: GPUComputableAutomaton>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipe_info: &PipelineInfo<A::Pipeline>,
        gpu_src: Arc<DeviceLocalBuffer<[u32]>>,
        gpu_dst: Arc<DeviceLocalBuffer<[u32]>>,
        push_constants: A::PushConstants,
        dim: &Dimensions,
    ) -> Self
    where
        A::Cell: Transcoder,
    {
        let cpu_out = unsafe {
            CpuAccessibleBuffer::uninitialized_array(
                Arc::clone(&device),
                dim.size() as usize,
                BufferUsage::all(),
                true,
            )
            .unwrap()
        };

        let set = Arc::new(
            PersistentDescriptorSet::start(Arc::clone(&pipe_info.layout))
                .add_buffer(Arc::clone(&gpu_src))
                .unwrap()
                .add_buffer(Arc::clone(&gpu_dst))
                .unwrap()
                .build()
                .unwrap(),
        );

        let cmd_exe = Arc::new(
            AutoCommandBufferBuilder::primary(Arc::clone(&device), queue.family())
                .unwrap()
                .dispatch(
                    [dim.width(), dim.height(), 1],
                    Arc::clone(&pipe_info.pipeline),
                    Arc::clone(&set),
                    push_constants,
                )
                .unwrap()
                .build()
                .unwrap(),
        );

        let cmd_cpy = Arc::new(
            AutoCommandBufferBuilder::primary(Arc::clone(&device), queue.family())
                .unwrap()
                .copy_buffer(Arc::clone(&gpu_dst), Arc::clone(&cpu_out))
                .unwrap()
                .build()
                .unwrap(),
        );

        Self {
            device,
            queue,
            cpu_out,
            cmd_exe,
            cmd_cpy,
        }
    }

    fn exe<F: GpuFuture>(&self, after: F) -> CommandBufferExecFuture<F, Arc<AutoCommandBuffer>> {
        after
            .then_execute(Arc::clone(&self.queue), Arc::clone(&self.cmd_exe))
            .unwrap()
    }

    fn cpy(&self) -> CommandBufferExecFuture<NowFuture, Arc<AutoCommandBuffer>> {
        sync::now(Arc::clone(&self.device))
            .then_execute(Arc::clone(&self.queue), Arc::clone(&self.cmd_cpy))
            .unwrap()
    }
}

pub struct CPUCompute<A: CPUComputableAutomaton> {
    grid: Grid<A::Cell>,
}

impl<A: CPUComputableAutomaton> CPUCompute<A> {
    pub fn new(initial_grid: Grid<A::Cell>) -> Self {
        Self { grid: initial_grid }
    }

    pub fn dispatch(
        mut self,
        rx_op: Receiver<ComputeOP<A>>,
        tx_data: Sender<GridHistoryOP<A::Cell>>,
    ) {
        loop {
            match rx_op.recv() {
                Ok(op) => match op {
                    ComputeOP::Reset(grid) => self.grid = grid,
                    ComputeOP::Run(nb_gens) => {
                        let mut grid = self.grid;
                        for _i in 0..nb_gens {
                            grid = A::update_grid(&grid);
                            if let Err(_) = tx_data.send(GridHistoryOP::Push(grid.clone())) {
                                break;
                            }
                        }
                        self.grid = grid;
                    }
                },
                Err(_) => break, // Sender died, time to die
            }
        }
    }
}

const ERR_NB_NODES: &str = "The number of compute nodes must be at least 2.";
