// Standard library
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

// External library
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture,
};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::sync::{self, GpuFuture, NowFuture};

// CELL
use super::{Neighbor2D, Position2D, Size2D};
use crate::advanced_channels::TransmittingEnd;
use crate::automaton::{AutomatonCell, CPUCell, GPUCell};
use crate::universe::{CPUUniverse, GPUUniverse, Universe, UniverseAutomatonShader, UniverseDiff, ShaderInfo};

pub struct Static2DGrid<C: AutomatonCell> {
    data: Vec<C>,
    size: Size2D,
    size_with_margin: Size2D,
    margin: usize,
    gpu: Option<GPUCompute<C>>,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Static2DGrid<C> {
    pub fn new(data: Vec<C>, size: Size2D) -> Self {
        if data.len() != size.total() {
            panic!("Vector length does not correspond to Size2D.")
        }

        // Determine the required margin around the actual data
        let margin = Static2DGrid::<C>::compute_margin();
        let size_with_margin = Size2D(size.0 + (margin << 1), size.1 + (margin << 1));

        // Create grid with margin
        let full_data = {
            let end_margins_len = margin * size_with_margin.0;
            let add_data_len = (margin * size_with_margin.1 + end_margins_len) * 2;

            let default_val = C::default();
            let mut full_data = Vec::with_capacity(data.len() + add_data_len);

            let push_n_default = |data: &mut Vec<C>, n: usize| {
                for _ in 0..n {
                    data.push(default_val.clone())
                }
            };

            // Fill in the new vector
            push_n_default(&mut full_data, end_margins_len);
            let mut data_iter = data.into_iter();
            for _ in 0..size.1 {
                push_n_default(&mut full_data, margin);
                for _ in 0..size.0 {
                    full_data.push(data_iter.next().unwrap());
                }
                push_n_default(&mut full_data, margin);
            }
            push_n_default(&mut full_data, end_margins_len);

            full_data
        };

        Self {
            data: full_data,
            size,
            size_with_margin,
            margin,
            gpu: None,
        }
    }

    pub fn encode(&self) -> Vec<C::Encoded> {
        let mut encoded = Vec::with_capacity(self.size.total());
        for cell in self.data.iter() {
            encoded.push(cell.encode());
        }
        encoded
    }

    pub fn decode(encoded: Arc<CpuAccessibleBuffer<[C::Encoded]>>, size: Size2D) -> Self {
        let margin = Static2DGrid::<C>::compute_margin();
        let size_with_margin = Size2D(size.0 + (margin << 1), size.1 + (margin << 1));
        let total_size = size_with_margin.total();

        // Decode data from CPU buffer
        let raw_data = encoded.read().unwrap();
        let mut decoded = Vec::with_capacity(total_size);
        for idx in 0..total_size {
            decoded.push(C::decode(&raw_data[idx]));
        }

        if decoded.len() != total_size {
            panic!(ERR_DECODED_SIZE);
        }

        Self {
            data: decoded,
            size,
            size_with_margin,
            margin,
            gpu: None,
        }
    }

    #[inline]
    pub fn size(&self) -> &Size2D {
        &self.size
    }

    #[inline]
    pub fn iter(&self) -> Static2DGridIterator<C> {
        Static2DGridIterator::new(self)
    }

    #[inline]
    fn get_unchecked(&self, idx: usize) -> &C {
        &self.data[idx]
    }

    fn move_grid_info(self, new_data: Vec<C>) -> Self {
        Self {
            data: new_data,
            size: self.size,
            size_with_margin: self.size_with_margin,
            margin: self.margin,
            gpu: self.gpu,
        }
    }

    fn compute_margin() -> usize {
        let mut max_manhattan_distance = 0;
        let neighbors = C::neighborhood();
        for n in neighbors {
            let x = n.0.abs() as usize;
            let y = n.1.abs() as usize;
            if x < y && max_manhattan_distance < y {
                max_manhattan_distance = y;
            } else if max_manhattan_distance < x {
                max_manhattan_distance = x;
            }
        }
        max_manhattan_distance
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Universe for Static2DGrid<C> {
    type Cell = C;
    type Position = Position2D;
    type Neighbor = Neighbor2D;
    type Diff = GridDiff<C>;

    fn get(&self, pos: Self::Position) -> &Self::Cell {
        let real_pos = Position2D(pos.0 + self.margin, pos.1 + self.margin);
        &self.data[real_pos.idx(&self.size_with_margin)]
    }

    fn neighbor(&self, pos: &Self::Position, nbor: &Self::Neighbor) -> &Self::Cell {
        let offset = nbor.0 + nbor.1 * self.size_with_margin.0 as i32;
        let idx = pos.idx(&self.size_with_margin);
        if offset < 0 {
            &self.data[idx - offset.abs() as usize]
        } else {
            &self.data[idx + offset as usize]
        }
    }

    fn diff(&self, other: &Self) -> Self::Diff {
        GridDiff::new(self, other)
    }

    fn apply_diff(self, diff: &Self::Diff) -> Self {
        let mut new_data = self.data.clone();
        for (idx, new_cell) in diff.iter() {
            new_data[*idx] = *new_cell
        }

        self.move_grid_info(new_data)
    }
}

impl<C: CPUCell<Neighbor = Neighbor2D>> CPUUniverse for Static2DGrid<C> {
    fn evolve_once(self) -> Self {
        // Compute new grid
        let mut new_data = Vec::with_capacity(self.data.len());
        for (pos, cell) in self.iter() {
            let new_cell = cell.update(&self, &pos);
            new_data.push(new_cell);
        }

        self.move_grid_info(new_data)
    }
}

impl<C: GPUCell<Neighbor = Neighbor2D>> Static2DGrid<C>
where
    Static2DGrid<C>: UniverseAutomatonShader<C>,
{
    fn get_gpu_handle(&mut self) -> &mut GPUCompute<C> {
        if let None = self.gpu {
            self.gpu = Some(GPUCompute::new(self, 16));
        }
        self.gpu.as_mut().unwrap()
    }
}

impl<C: GPUCell<Neighbor = Neighbor2D>> GPUUniverse for Static2DGrid<C>
where
    Static2DGrid<C>: UniverseAutomatonShader<C>,
{
    fn evolve(mut self, nb_gens: usize) -> Self {
        self.get_gpu_handle().run(nb_gens)
    }

    fn evolve_mailbox<T: TransmittingEnd<MSG = Self>>(
        mut self,
        nb_gens: usize,
        mailbox: &T,
    ) -> Self {
        self.get_gpu_handle().run_mailbox(nb_gens, mailbox)
    }
}

impl<C: AutomatonCell> Clone for Static2DGrid<C> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            size: self.size,
            size_with_margin: self.size_with_margin,
            margin: self.margin,
            gpu: None,
        }
    }
}

pub struct Static2DGridIterator<'a, C: AutomatonCell> {
    grid: &'a Static2DGrid<C>,
    pos: Position2D,
    idx: usize,
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> Static2DGridIterator<'a, C> {
    fn new(grid: &'a Static2DGrid<C>) -> Self {
        Self {
            grid,
            pos: Position2D(0, 0),
            idx: grid.margin * grid.size_with_margin.0 + grid.margin,
        }
    }
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> Iterator for Static2DGridIterator<'a, C> {
    type Item = (Position2D, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        let idx = self.idx;

        // Update pos and idx
        if self.pos.1 == self.grid.size.1 {
            return None;
        } else if self.pos.0 == self.grid.size.0 - 1 {
            self.pos.0 = 0;
            self.pos.1 += 1;
            self.idx += 2 * self.grid.margin + 1;
        } else {
            self.pos.0 += 1;
            self.idx += 1;
        }

        let cell = self.grid.get_unchecked(idx);
        Some((pos, cell))
    }
}

#[derive(Debug, Clone)]
pub struct GridDiff<C: AutomatonCell> {
    modifs: HashMap<usize, C>,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> GridDiff<C> {
    pub fn new(prev_grid: &Static2DGrid<C>, next_grid: &Static2DGrid<C>) -> Self {
        let size = prev_grid.size();
        if size != next_grid.size() {
            panic!("Both grids should be the same dimensions!")
        }

        let mut modifs = HashMap::new();
        for idx in 0..size.total() {
            let prev = prev_grid.get_unchecked(idx);
            let next = next_grid.get_unchecked(idx);
            if prev != next {
                modifs.insert(idx, *next);
            }
        }

        Self { modifs }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&usize, &C)> {
        self.modifs.iter()
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> UniverseDiff for GridDiff<C> {
    fn no_diff() -> Self {
        Self {
            modifs: HashMap::new(),
        }
    }

    fn stack(&mut self, other: &Self) {
        for (pos, new_cell) in other.modifs.iter() {
            match self.modifs.get_mut(pos) {
                Some(old_cell) => *old_cell = *new_cell,
                None => {
                    self.modifs.insert(*pos, *new_cell);
                }
            }
        }
    }
}

#[derive(Clone)]
struct GPUCompute<C: AutomatonCell> {
    size: Size2D,
    device: Arc<Device>,
    queue: Arc<Queue>,
    nodes: Vec<ComputeNode<C>>,
    next: usize,
}

impl<C: GPUCell<Neighbor = Neighbor2D>> GPUCompute<C>
where
    Static2DGrid<C>: UniverseAutomatonShader<C>,
{
    fn new(grid: &Static2DGrid<C>, nb_nodes: usize) -> Self {
        // Create a logical device and compute queue
        let (device, queue) = {
            // Create a Vulkan instance and physical device
            let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
            let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

            // Select a queue family from the physical device
            let comp_q_family = physical
                .queue_families()
                .find(|&q| q.supports_compute())
                .unwrap();

            // Create a logical device and retreive the compute queue handle
            let (device, mut queues) = Device::new(
                physical,
                physical.supported_features(),
                &DeviceExtensions {
                    khr_storage_buffer_storage_class: true,
                    ..DeviceExtensions::none()
                },
                [(comp_q_family, 0.5)].iter().cloned(),
            )
            .unwrap();
            (device, queues.next().unwrap())
        };

        // Create GPU buffers
        let gpu_bufs = {
            if nb_nodes < 2 {
                panic!(ERR_NB_NODES)
            }
            let mut gpu_bufs = Vec::with_capacity(nb_nodes);
            let total_size = grid.size_with_margin.total();
            for _ in 0..nb_nodes {
                let q_family = vec![queue.family()];
                let buf: Arc<DeviceLocalBuffer<[C::Encoded]>> = DeviceLocalBuffer::array(
                    Arc::clone(&device),
                    total_size,
                    BufferUsage::all(),
                    q_family,
                )
                .unwrap();
                gpu_bufs.push(buf)
            }
            gpu_bufs
        };

        // Create compute nodes
        let nodes = {
            let mut nodes = Vec::with_capacity(nb_nodes);
            for i in 0..nb_nodes {
                let j = {
                    if i == nb_nodes - 1 {
                        0
                    } else {
                        i + 1
                    }
                };

                let shader = Static2DGrid::shader_info(&device);
                nodes.push(ComputeNode::new(
                    grid,
                    &shader,
                    Arc::clone(&device),
                    Arc::clone(&queue),
                    Arc::clone(&gpu_bufs[i]),
                    Arc::clone(&gpu_bufs[j]),
                ))
            }
            nodes
        };

        // Copy grid to first GPU buffer
        {
            let cpu_buf = CpuAccessibleBuffer::from_iter(
                Arc::clone(&device),
                BufferUsage::transfer_source(),
                false,
                grid.encode().into_iter(),
            )
            .unwrap();
            let cmd = AutoCommandBufferBuilder::primary_one_time_submit(
                Arc::clone(&device),
                queue.family(),
            )
            .unwrap()
            .copy_buffer(cpu_buf, gpu_bufs[0].clone())
            .unwrap()
            .build()
            .unwrap();
            sync::now(Arc::clone(&device))
                .then_execute(Arc::clone(&queue), cmd)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();
        }

        // Create and store new GPUCompute instance
        Self {
            size: grid.size,
            device,
            queue,
            nodes,
            next: 0,
        }
    }

    fn run(&mut self, nb_gens: usize) -> Static2DGrid<C> {
        // Total number of compute nodes
        let nb_nodes = self.nodes.len();

        // Update next pointer for further calls to run()
        let start_node = self.next;
        self.next = (self.next + nb_gens) % nb_nodes;

        // Chain nb_gens execution command buffers and copy back data from last node
        let mut next_exe_node = start_node;
        let cpy_node = (start_node + nb_gens - 1) % nb_nodes;
        let mut future = Box::new(sync::now(Arc::clone(&self.device))) as Box<dyn GpuFuture>;
        for _i in 0..nb_gens {
            future = Box::new(self.nodes[next_exe_node].exe(future));
            next_exe_node = self.wrap_ptr(next_exe_node)
        }
        future = Box::new(self.nodes[cpy_node].cpy_after(future));
        GPUCompute::<C>::wait_for_future(future);

        let encoded = Arc::clone(&self.nodes[cpy_node].cpu_out);
        Static2DGrid::decode(encoded, self.size)
    }

    fn run_mailbox(
        &mut self,
        nb_gens: usize,
        mailbox: &impl TransmittingEnd<MSG = Static2DGrid<C>>,
    ) -> Static2DGrid<C> {
        // Total number of compute nodes
        let nb_nodes = self.nodes.len();

        let min = |a, b| {
            if a < b {
                a
            } else {
                b
            }
        };

        let now_future = |device| Box::new(sync::now(device)) as Box<dyn GpuFuture>;

        // Update next pointer for further calls to run()
        let start_node = self.next;
        self.next = (self.next + nb_gens) % nb_nodes;

        // Countdown on number of generations that must still be computed
        // (i.e., number of exe futures left to be scheduled)
        let mut gens_to_compute = nb_gens;

        // Number of execution futures chained together in exe_future
        let mut launch_cnt = min(nb_nodes, gens_to_compute);

        // Launch command buffers
        let mut next_exe_node = start_node;
        let mut exe_future = now_future(Arc::clone(&self.device));
        for _i in 0..launch_cnt {
            exe_future = Box::new(self.nodes[next_exe_node].exe(exe_future));
            next_exe_node = self.wrap_ptr(next_exe_node)
        }
        gens_to_compute -= launch_cnt;

        let mut cpy_futures = VecDeque::with_capacity(launch_cnt);

        loop {
            // Wait for all nodes to finish execution
            GPUCompute::<C>::wait_for_future(exe_future);

            // Tell all compute nodes to bring back data to CPU
            let mut next_cpy_node = start_node;
            for _i in 0..launch_cnt {
                cpy_futures.push_back((self.nodes[next_cpy_node].cpy(), next_cpy_node));
                next_cpy_node = self.wrap_ptr(next_cpy_node);
            }

            // Start reading back data and re-launch computations as needed
            launch_cnt = min(nb_nodes, gens_to_compute);
            exe_future = now_future(Arc::clone(&self.device));
            loop {
                match cpy_futures.pop_front() {
                    Some((future, idx)) => {
                        // Wait for the copy operation to complete
                        GPUCompute::<C>::wait_for_future(Box::new(future));

                        // This node is available for compute again
                        if gens_to_compute > 0 {
                            exe_future = Box::new(self.nodes[idx].exe(exe_future));
                            gens_to_compute -= 1;
                        }

                        // Transform raw data into Grid and send to mailbox
                        let encoded = Arc::clone(&self.nodes[idx].cpu_out);
                        let new_grid = Static2DGrid::decode(encoded, self.size);
                        if launch_cnt == 0 && cpy_futures.len() == 0 {
                            mailbox.send(new_grid.clone());
                            return new_grid;
                        } else {
                            mailbox.send(new_grid);
                        }
                    }
                    None => break,
                }
            }
        }
    }

    fn wait_for_future(future: Box<dyn GpuFuture>) {
        future
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap()
    }

    fn wrap_ptr(&self, ptr: usize) -> usize {
        if ptr == self.nodes.len() - 1 {
            0
        } else {
            ptr + 1
        }
    }
}

#[derive(Clone)]
struct ComputeNode<C: AutomatonCell> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    cpu_out: Arc<CpuAccessibleBuffer<[C::Encoded]>>,
    cmd_exe: Arc<AutoCommandBuffer>,
    cmd_cpy: Arc<AutoCommandBuffer>,
}

impl<C: AutomatonCell> ComputeNode<C> {
    fn new(
        grid: &Static2DGrid<C>,
        shader: &ShaderInfo,
        device: Arc<Device>,
        queue: Arc<Queue>,
        gpu_src: Arc<DeviceLocalBuffer<[C::Encoded]>>,
        gpu_dst: Arc<DeviceLocalBuffer<[C::Encoded]>>,
    ) -> Self {
        let total_size = grid.size_with_margin.total();
        let pc = PushConstants {
            width: grid.size.0 as u32,
            height: grid.size.1 as u32,
            margin: grid.margin as u32,
        };

        // CPU buffer to pull data out of GPU
        let cpu_out = unsafe {
            CpuAccessibleBuffer::uninitialized_array(
                Arc::clone(&device),
                total_size,
                BufferUsage::all(),
                true,
            )
            .unwrap()
        };

        // Descriptor set
        let set = Arc::new(
            PersistentDescriptorSet::start(Arc::clone(&shader.layout))
                .add_buffer(Arc::clone(&gpu_src))
                .unwrap()
                .add_buffer(Arc::clone(&gpu_dst))
                .unwrap()
                .build()
                .unwrap(),
        );

        // TODO add neighbor buffer

        // Run shader command
        let cmd_exe = Arc::new(
            AutoCommandBufferBuilder::primary(Arc::clone(&device), queue.family())
                .unwrap()
                .dispatch(
                    [
                        grid.size_with_margin.0 as u32,
                        grid.size_with_margin.1 as u32,
                        1,
                    ],
                    Arc::clone(&shader.pipeline),
                    Arc::clone(&set),
                    pc,
                )
                .unwrap()
                .build()
                .unwrap(),
        );

        // CPU writeback command
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

    fn cpy_after<F: GpuFuture>(
        &self,
        after: F,
    ) -> CommandBufferExecFuture<F, Arc<AutoCommandBuffer>> {
        after
            .then_execute(Arc::clone(&self.queue), Arc::clone(&self.cmd_cpy))
            .unwrap()
    }
}

#[repr(C)]
struct PushConstants {
    width: u32,
    height: u32,
    margin: u32,
}

const ERR_NB_NODES: &str = "The number of compute nodes should be strictly greater than 1.";
const ERR_DECODED_SIZE: &str =
    "The size of decoded data doesn't correspond to the indicated grid size.";
