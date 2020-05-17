// Standard library
use std::sync::{mpsc, Arc};

// External libraries
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::pipeline::ComputePipelineAbstract;
use vulkano::sync::{self, GpuFuture, NowFuture};

// CELL
use super::{ComputeOP, PipelineInfo};
use crate::grid::Dimensions;

pub struct ComputeCluster<P: ComputePipelineAbstract + Send + Sync + 'static> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipe_info: PipelineInfo<P>,
    gpu_bufs: Vec<Arc<DeviceLocalBuffer<[u32]>>>,
    nodes: Vec<ComputeNode>,
    next_exe: usize,
    next_cpy: usize,
    pending_cpy: bool,
}

impl<P: ComputePipelineAbstract + Send + Sync + 'static> ComputeCluster<P> {
    pub fn new<C: Copy>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipe_info: PipelineInfo<P>,
        push_constants: C,
        nb_nodes: usize,
        dim: &Dimensions,
    ) -> Self {
        if nb_nodes == 0 {
            panic!("The number of compute nodes must be strictly positive.")
        }

        let total_size = dim.size() as usize;

        let mut gpu_bufs = Vec::with_capacity(nb_nodes);
        for _ in 0..nb_nodes {
            let q_family = vec![queue.family()];
            gpu_bufs.push(
                DeviceLocalBuffer::array(device.clone(), total_size , BufferUsage::all(), q_family)
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
            nodes.push(ComputeNode::new(
                Arc::clone(&device),
                Arc::clone(&queue),
                &pipe_info,
                Arc::clone(&gpu_bufs[i]),
                Arc::clone(&gpu_bufs[j]),
                push_constants,
                dim,
            ))
        }

        Self {
            device,
            queue,
            pipe_info,
            gpu_bufs,
            nodes,
            next_exe: 0,
            next_cpy: 0,
            pending_cpy: false,
        }
    }

    pub fn dispatch(
        mut self,
        rx_op: mpsc::Receiver<ComputeOP>,
        tx_data: mpsc::Sender<Vec<Arc<CpuAccessibleBuffer<[u32]>>>>,
    ) {
        loop {
            match rx_op.recv() {
                Ok(op) => match op {
                    ComputeOP::Reset(data) => self.reset(data),
                    ComputeOP::Run(nb_gens) => self.run(nb_gens, &tx_data),
                },
                Err(_) => break, // Time to die
            }
        }
    }

    fn reset(&mut self, data: Vec<u32>) {
        // Reset pointers
        self.next_exe = 0;
        self.next_cpy = 0;
        self.pending_cpy = false;

        // Put data in first GPU buffer
        let cpu_buf = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::transfer_source(),
            false,
            data.into_iter(),
        )
        .unwrap();
        let cmd = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .copy_buffer(cpu_buf, self.gpu_bufs[0].clone())
        .unwrap()
        .build()
        .unwrap();
        sync::now(self.device.clone())
            .then_execute(self.queue.clone(), cmd)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }

    fn run(&mut self, nb_gens: u64, tx_data: &mpsc::Sender<Vec<Arc<CpuAccessibleBuffer<[u32]>>>>) {
        // Total number of compute nodes
        let nb_nodes = self.nodes.len();

        // Countdown on number of generations that must still be computed
        let mut gens_to_compute = nb_gens;

        while gens_to_compute > 0 {
            // Returns the number of compute nodes available
            let check_available_ressources = || {
                if !self.pending_cpy {
                    nb_nodes
                } else if self.next_cpy < self.next_exe {
                    nb_nodes - self.next_exe + self.next_cpy
                } else {
                    self.next_cpy - self.next_exe
                }
            };

            let mut nb_available = check_available_ressources();
            while nb_available == 0 {
                // @TODO: do something here
                nb_available = check_available_ressources();
            }

            // We have some computing nodes available, launch computations on those
            let launch_cnt = {
                if (nb_available as u64) < gens_to_compute {
                    nb_available as u64
                } else {
                    gens_to_compute
                }
            };

            // Launch command buffers
            let mut compute_future = Box::new(sync::now(self.device.clone())) as Box<dyn GpuFuture>;
            for _i in 0..launch_cnt {
                // Chain futures
                compute_future = Box::new(self.nodes[self.next_exe].exe(compute_future));

                // Increment pointer to next execution units
                self.next_exe = {
                    if self.next_exe == nb_nodes - 1 {
                        0
                    } else {
                        self.next_exe + 1
                    }
                }
            }

            compute_future
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();

            gens_to_compute -= launch_cnt;
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
    fn new<T: ComputePipelineAbstract + Send + Sync + 'static, C>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipe_info: &PipelineInfo<T>,
        gpu_src: Arc<DeviceLocalBuffer<[u32]>>,
        gpu_dst: Arc<DeviceLocalBuffer<[u32]>>,
        push_constants: C,
        dim: &Dimensions,
    ) -> Self {
        let cpu_out = unsafe {
            CpuAccessibleBuffer::uninitialized_array(
                device.clone(),
                dim.size() as usize,
                BufferUsage::all(),
                true,
            )
            .unwrap()
        };

        let set = Arc::new(
            PersistentDescriptorSet::start(pipe_info.layout.clone())
                .add_buffer(gpu_src.clone())
                .unwrap()
                .add_buffer(gpu_dst.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let cmd_exe = Arc::new(
            AutoCommandBufferBuilder::primary(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [dim.width(), dim.height(), 1],
                    pipe_info.pipeline.clone(),
                    set.clone(),
                    push_constants,
                )
                .unwrap()
                .build()
                .unwrap(),
        );

        let cmd_cpy = Arc::new(
            AutoCommandBufferBuilder::primary(device.clone(), queue.family())
                .unwrap()
                .copy_buffer(gpu_dst.clone(), cpu_out.clone())
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
            .then_execute(self.queue.clone(), self.cmd_exe.clone())
            .unwrap()
    }

    fn cpy(&self) -> CommandBufferExecFuture<NowFuture, Arc<AutoCommandBuffer>> {
        sync::now(self.device.clone())
            .then_execute(self.queue.clone(), self.cmd_cpy.clone())
            .unwrap()
    }
}
