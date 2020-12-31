// Standard library
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

// External library
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture},
    descriptor::descriptor_set::PersistentDescriptorSet,
    device::{Device, DeviceExtensions, Queue},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
    sync::{self, GpuFuture, NowFuture},
};

// CELL
use super::{Coordinates2D, Neighbor2D, Size2D};
use crate::{
    automaton::{AutomatonCell, CPUCell, GPUCell},
    universe::{
        CPUUniverse, GPUUniverse, GenerationDifference, ShaderInfo, Universe,
        UniverseAutomatonShader,
    },
};

const DISPATCH_LAYOUT: (usize, usize, usize) = (8, 8, 1);

/// StaticGrid2D

pub struct StaticGrid2D<C: AutomatonCell> {
    data: Vec<C>,
    size: Size2D,
    size_with_margin: Size2D,
    margin: usize,
    gpu: Option<GPUCompute<C>>,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> StaticGrid2D<C> {
    pub fn new(data: Vec<C>, size: Size2D) -> Self {
        if data.len() != size.total() {
            panic!(ERR_DIMENSIONS_SIZE)
        }

        // Determine the required margin around the actual data
        let margin = Neighbor2D::max_one_axis_manhattan_distance(C::neighborhood());
        let size_with_margin = Size2D(size.columns() + (margin << 1), size.lines() + (margin << 1));

        // Create grid with margin
        let full_data = {
            let end_margins_len = margin * size_with_margin.columns();
            let add_data_len = (margin * size_with_margin.lines() + end_margins_len) * 2;

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
            for _ in 0..size.lines() {
                push_n_default(&mut full_data, margin);
                for _ in 0..size.columns() {
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

    pub fn new_empty(size: Size2D) -> Self {
        // Determine the required margin around the actual data
        let margin = Neighbor2D::max_one_axis_manhattan_distance(C::neighborhood());
        let size_with_margin = Size2D(size.columns() + (margin << 1), size.lines() + (margin << 1));

        Self {
            data: vec![C::default(); size_with_margin.total()],
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
        let margin = Neighbor2D::max_one_axis_manhattan_distance(C::neighborhood());
        let size_with_margin = Size2D(size.columns() + (margin << 1), size.lines() + (margin << 1));
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
    pub fn iter(&self) -> StaticGrid2DIterator<C> {
        StaticGrid2DIterator::new(self)
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> Universe for StaticGrid2D<C> {
    type Cell = C;
    type Coordinates = Coordinates2D;

    fn get(&self, coords: Self::Coordinates) -> Self::Cell {
        let real_coords = Coordinates2D(coords.x() + self.margin, coords.y() + self.margin);
        self.data[real_coords.to_idx(&self.size_with_margin)]
    }

    fn set(&mut self, coords: Self::Coordinates, val: Self::Cell) {
        let real_coords = Coordinates2D(coords.x() + self.margin, coords.y() + self.margin);
        self.data[real_coords.to_idx(&self.size_with_margin)] = val;
    }
    fn neighbor(
        &self,
        coords: Self::Coordinates,
        nbor: <Self::Cell as AutomatonCell>::Neighbor,
    ) -> Self::Cell {
        let real_coords = Coordinates2D(coords.x() + self.margin, coords.y() + self.margin);
        let mut idx = real_coords.to_idx(&self.size_with_margin);
        if nbor.x() < 0 {
            idx -= nbor.x().abs() as usize
        } else {
            idx += nbor.x() as usize
        }
        if nbor.y() < 0 {
            idx -= nbor.y().abs() as usize * self.size_with_margin.columns()
        } else {
            idx += nbor.y() as usize * self.size_with_margin.columns()
        }
        self.data[idx]
    }
}

impl<C: CPUCell<Neighbor = Neighbor2D>> CPUUniverse for StaticGrid2D<C> {
    fn cpu_evolve_once(mut self) -> Self {
        // Compute new grid
        let mut new_data = vec![C::default(); self.size_with_margin.total()];
        for col_iter in self.iter() {
            for (coords, cell) in col_iter {
                let new_cell = cell.update(&self, coords);
                let real_coords = Coordinates2D(coords.x() + self.margin, coords.y() + self.margin);
                new_data[real_coords.to_idx(&self.size_with_margin)] = new_cell;
            }
        }

        self.data = new_data;
        self
    }
}

impl<C: GPUCell<Neighbor = Neighbor2D>> StaticGrid2D<C>
where
    StaticGrid2D<C>: UniverseAutomatonShader<C>,
{
    fn get_gpu_handle(&mut self) -> &mut GPUCompute<C> {
        if let None = self.gpu {
            self.gpu = Some(GPUCompute::new(self, 16));
        }
        self.gpu.as_mut().unwrap()
    }
}

impl<C: GPUCell<Neighbor = Neighbor2D>> GPUUniverse for StaticGrid2D<C>
where
    StaticGrid2D<C>: UniverseAutomatonShader<C>,
{
    fn gpu_evolve(mut self, nb_gens: usize) -> Self {
        self.get_gpu_handle().run(nb_gens)
    }

    fn gpu_evolve_callback(mut self, nb_gens: usize, callback: impl Fn(&Self)) -> Self {
        self.get_gpu_handle().run_mailbox(nb_gens, callback)
    }
}

impl<C: AutomatonCell> Clone for StaticGrid2D<C> {
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

/// StaticGrid2DIterator

pub struct StaticGrid2DIterator<'a, C: AutomatonCell> {
    grid: &'a StaticGrid2D<C>,
    line_idx: usize,
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> StaticGrid2DIterator<'a, C> {
    fn new(grid: &'a StaticGrid2D<C>) -> Self {
        Self { grid, line_idx: 0 }
    }
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> Iterator for StaticGrid2DIterator<'a, C> {
    type Item = StaticGrid2DLineIterator<'a, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line_idx < self.grid.size.lines() {
            let col_iterator = StaticGrid2DLineIterator::new(self.grid, self.line_idx);
            self.line_idx += 1;
            Some(col_iterator)
        } else {
            None
        }
    }
}

/// StaticGrid2DLineIterator

pub struct StaticGrid2DLineIterator<'a, C: AutomatonCell> {
    grid: &'a StaticGrid2D<C>,
    coords: Coordinates2D,
    idx: usize,
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> StaticGrid2DLineIterator<'a, C> {
    pub fn new(grid: &'a StaticGrid2D<C>, line_idx: usize) -> Self {
        Self {
            grid,
            coords: Coordinates2D(0, line_idx),
            idx: (line_idx + grid.margin) * grid.size_with_margin.columns() + grid.margin,
        }
    }
}

impl<'a, C: AutomatonCell<Neighbor = Neighbor2D>> Iterator for StaticGrid2DLineIterator<'a, C> {
    type Item = (Coordinates2D, C);

    fn next(&mut self) -> Option<Self::Item> {
        if self.coords.x() < self.grid.size.columns() {
            let ret_coords = self.coords;
            let cell = self.grid.data[self.idx];
            self.coords.0 += 1;
            self.idx += 1;
            Some((ret_coords, cell))
        } else {
            None
        }
    }
}

/// GridDiff

#[derive(Debug, Clone)]
pub struct GridDiff<C: AutomatonCell> {
    modifs: HashMap<usize, C>,
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> GridDiff<C> {
    pub fn iter(&self) -> impl Iterator<Item = (&usize, &C)> {
        self.modifs.iter()
    }
}

impl<C: AutomatonCell<Neighbor = Neighbor2D>> GenerationDifference for GridDiff<C> {
    type Universe = StaticGrid2D<C>;

    fn get_diff(base: &Self::Universe, target: &Self::Universe) -> Self {
        if base.size() != target.size() {
            panic!(ERR_WRONG_DIMENSIONS)
        }

        let mut modifs = HashMap::new();
        for idx in 0..base.data.len() {
            let prev = &base.data[idx];
            let next = &target.data[idx];
            if prev != next {
                modifs.insert(idx, *next);
            }
        }

        Self { modifs }
    }

    fn apply_to(&self, mut base: Self::Universe) -> Self::Universe {
        let mut new_data = base.data.clone();
        for (idx, new_cell) in self.iter() {
            new_data[*idx] = *new_cell
        }

        base.data = new_data;
        base
    }

    fn empty_diff() -> Self {
        Self {
            modifs: HashMap::new(),
        }
    }

    fn stack(&mut self, other: &Self) {
        for (coords, new_cell) in other.modifs.iter() {
            match self.modifs.get_mut(coords) {
                Some(old_cell) => *old_cell = *new_cell,
                None => {
                    self.modifs.insert(*coords, *new_cell);
                }
            }
        }
    }
}

/// GPUCompute

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
    StaticGrid2D<C>: UniverseAutomatonShader<C>,
{
    fn new(grid: &StaticGrid2D<C>, nb_nodes: usize) -> Self {
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

                let shader = StaticGrid2D::shader_info(&device);
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

    fn run(&mut self, nb_gens: usize) -> StaticGrid2D<C> {
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
        Self::wait_for_future(future);

        let encoded = Arc::clone(&self.nodes[cpy_node].cpu_out);
        StaticGrid2D::decode(encoded, self.size)
    }

    fn run_mailbox(
        &mut self,
        nb_gens: usize,
        callback: impl Fn(&StaticGrid2D<C>) -> (),
    ) -> StaticGrid2D<C> {
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
            Self::wait_for_future(exe_future);

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
                        Self::wait_for_future(Box::new(future));

                        // This node is available for compute again
                        if gens_to_compute > 0 {
                            exe_future = Box::new(self.nodes[idx].exe(exe_future));
                            gens_to_compute -= 1;
                        }

                        // Transform raw data into Grid and send to mailbox
                        let encoded = Arc::clone(&self.nodes[idx].cpu_out);
                        let new_grid = StaticGrid2D::decode(encoded, self.size);
                        callback(&new_grid);
                        if launch_cnt == 0 && cpy_futures.len() == 0 {
                            return new_grid;
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

/// ComputeNode

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
        grid: &StaticGrid2D<C>,
        shader: &ShaderInfo,
        device: Arc<Device>,
        queue: Arc<Queue>,
        gpu_src: Arc<DeviceLocalBuffer<[C::Encoded]>>,
        gpu_dst: Arc<DeviceLocalBuffer<[C::Encoded]>>,
    ) -> Self {
        let total_size = grid.size_with_margin.total();
        let pc = PushConstants {
            width: grid.size.columns() as u32,
            height: grid.size.lines() as u32,
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

        let dimensions = {
            let mut dimensions_x = grid.size.columns() / DISPATCH_LAYOUT.0;
            if dimensions_x * DISPATCH_LAYOUT.0 != grid.size.columns() {
                dimensions_x += 1;
            }
            let mut dimensions_y = grid.size.lines() / DISPATCH_LAYOUT.0;
            if dimensions_y * DISPATCH_LAYOUT.0 != grid.size.lines() {
                dimensions_y += 1;
            }
            [dimensions_x as u32, dimensions_y as u32, 0]
        };

        // Run shader command
        let cmd_exe = Arc::new(
            AutoCommandBufferBuilder::primary(Arc::clone(&device), queue.family())
                .unwrap()
                .dispatch(
                    dimensions,
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

/// PushConstants

#[repr(C)]
struct PushConstants {
    width: u32,
    height: u32,
    margin: u32,
}

const ERR_NB_NODES: &str = "The number of compute nodes should be strictly greater than 1.";
const ERR_DECODED_SIZE: &str =
    "The size of decoded data doesn't correspond to the indicated grid size.";
const ERR_WRONG_DIMENSIONS: &str = "Both grids should be the same dimensions!";
const ERR_DIMENSIONS_SIZE: &str = "Vector length does not correspond to Size2D.";
