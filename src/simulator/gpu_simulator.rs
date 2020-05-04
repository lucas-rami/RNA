// Standard library
use std::sync::{mpsc, Arc};

// External libraries
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::{
    DescriptorSetsCollection, PersistentDescriptorSet, UnsafeDescriptorSetLayout,
};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::sync::{self, GpuFuture};

// CELL
use super::grid::{Dimensions, Grid, Position};
use super::{CellularAutomaton, Simulator};

pub trait GPUComputableAutomaton: CellularAutomaton {
    fn id_from_state(&self, state: &Self::State) -> u32;
    fn state_from_id(&self, id: u32) -> Self::State;

    fn bind_device(&mut self, device: &Arc<Device>) -> ();
    fn gpu_layout(&self) -> &Arc<UnsafeDescriptorSetLayout>;
    fn gpu_dispatch<T>(
        &self,
        cmd_buffer: AutoCommandBufferBuilder<T>,
        dispatch_dim: [u32; 3],
        sets: impl DescriptorSetsCollection,
        grid_dim: &Dimensions,
    ) -> AutoCommandBufferBuilder<T>;
}

pub struct GPUSimulator<A: GPUComputableAutomaton> {
    name: String,
    automaton: A,
    grid: Grid<A::State>,
    current_gen: u64,
    manager: ComputeManager,
}

impl<A: GPUComputableAutomaton> GPUSimulator<A> {
    pub fn new(
        name: &str,
        mut automaton: A,
        grid: &Grid<A::State>,
        instance: Arc<Instance>,
    ) -> Self {
        let manager = {
            // Select a queue family from the physical device
            let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
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
            let queue = queues.next().unwrap();

            // Bind the automaton to the device
            automaton.bind_device(&device);

            ComputeManager::new(device.clone(), queue, 4, &automaton, grid.dim())
        };

        Self {
            name: String::from(name),
            automaton,
            grid: grid.clone(),
            current_gen: 0,
            manager,
        }
    }

    fn grid_to_raw(&self) -> Vec<u32> {
        let dim = self.size();
        let size = dim.nb_elems();
        let mut raw_data = Vec::with_capacity(size);
        for state in self.grid.iter() {
            raw_data.push(self.automaton.id_from_state(state));
        }
        raw_data
    }

    fn raw_to_grid(&self, cpu_buffer: Arc<CpuAccessibleBuffer<[u32]>>) -> Vec<A::State> {
        let dim = self.size();
        let size = dim.nb_elems();
        let raw_data = cpu_buffer.read().unwrap();
        let mut grid = Vec::with_capacity(size);
        for i in 0..size {
            // println!("{}", raw_data[i]);
            grid.push(self.automaton.state_from_id(raw_data[i]));
        }
        grid
    }
}

impl<A: GPUComputableAutomaton> Simulator<A> for GPUSimulator<A> {
    fn run(&mut self, nb_gens: u64) -> () {
        self.current_gen += nb_gens;
    }

    fn automaton(&self) -> &A {
        &self.automaton
    }

    fn cell(&self, pos: &Position) -> &A::State {
        self.grid.get(pos)
    }

    fn size(&self) -> &Dimensions {
        self.grid.dim()
    }

    fn name(&self) -> &str {
        &self.name[..]
    }

    fn current_gen(&self) -> u64 {
        self.current_gen
    }
}

struct ComputeManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    gpu_bufs: Vec<Arc<DeviceLocalBuffer<[u32]>>>,
    comp_units: Vec<ComputeUnit>,
    next_exec: usize,
    next_copy: usize, 
}

impl ComputeManager {
    fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        nb_comp_units: usize,
        automaton: &(impl CellularAutomaton + GPUComputableAutomaton),
        size: &Dimensions,
    ) -> Self {
        let total_size = size.nb_elems();

        let mut gpu_bufs = Vec::with_capacity(nb_comp_units);
        for _ in 0..nb_comp_units {
            let q_family = vec![queue.family()];
            gpu_bufs.push(
                DeviceLocalBuffer::array(device.clone(), total_size, BufferUsage::all(), q_family)
                    .unwrap(),
            )
        }

        let mut comp_units = Vec::with_capacity(nb_comp_units);
        for i in 0..nb_comp_units {
            let j = {
                if i + 1 < nb_comp_units {
                    i + 1
                } else {
                    0
                }
            };
            comp_units.push(ComputeUnit::new(
                device.clone(),
                Arc::clone(&queue),
                Arc::clone(&gpu_bufs[i]),
                Arc::clone(&gpu_bufs[j]),
                automaton,
                size,
            ))
        }

        Self {
            device,
            queue,
            gpu_bufs,
            comp_units,
            next_exec: 0,
            next_copy: 0,
        }
    }

    fn run(&self, nb_gens: u64) -> () {

    }
}

struct ComputeUnit {
    device: Arc<Device>,
    queue: Arc<Queue>,
    cpu_out: Arc<CpuAccessibleBuffer<[u32]>>,
    cmd_exec: AutoCommandBuffer,
    cmd_copy: AutoCommandBuffer,
}

impl ComputeUnit {
    fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        gpu_src: Arc<DeviceLocalBuffer<[u32]>>,
        gpu_dst: Arc<DeviceLocalBuffer<[u32]>>,
        automaton: &(impl CellularAutomaton + GPUComputableAutomaton),
        size: &Dimensions,
    ) -> Self {
        let cpu_out = unsafe {
            CpuAccessibleBuffer::uninitialized_array(
                device.clone(),
                size.nb_elems(),
                BufferUsage::all(),
                true,
            )
            .unwrap()
        };

        let set = Arc::new(
            PersistentDescriptorSet::start(automaton.gpu_layout().clone())
                .add_buffer(gpu_src.clone())
                .unwrap()
                .add_buffer(gpu_dst.clone())
                .unwrap()
                .build()
                .unwrap(),
        );
        let cmd_exec = AutoCommandBufferBuilder::primary(device.clone(), queue.family()).unwrap();
        let cmd_exec = automaton
            .gpu_dispatch(
                cmd_exec,
                [size.nb_cols as u32, size.nb_rows as u32, 1],
                set,
                &size,
            )
            .build()
            .unwrap();

        let cmd_copy = AutoCommandBufferBuilder::primary(device.clone(), queue.family())
            .unwrap()
            .copy_buffer(gpu_dst.clone(), cpu_out.clone())
            .unwrap()
            .build()
            .unwrap();

        Self {
            device,
            queue,
            cpu_out,
            cmd_exec,
            cmd_copy,
        }
    }

    fn exec(&self) -> () {
        self.submit_and_wait(self.cmd_exec);
    }

    fn copy(&self) -> &Arc<CpuAccessibleBuffer<[u32]>> {
        self.submit_and_wait(self.cmd_copy);
        &self.cpu_out
    }

    fn submit_and_wait(&self, cmd: AutoCommandBuffer) -> () {
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), cmd)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
    }
}
