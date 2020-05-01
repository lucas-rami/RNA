// Standard library
use std::sync::Arc;

// External libraries
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
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
        dispatch_dim: [u32; 3],
        cmd_buffer: AutoCommandBufferBuilder<T>,
        sets: impl DescriptorSetsCollection,
        grid_dim: &Dimensions
    ) -> AutoCommandBufferBuilder<T>;
}

pub struct GPUSimulator<A: GPUComputableAutomaton> {
    name: String,
    automaton: A,
    grid: Grid<A::State>,
    current_gen: u64,
    vk: VKResources,
}

impl<A: GPUComputableAutomaton> GPUSimulator<A> {
    pub fn new(
        name: &str,
        mut automaton: A,
        grid: &Grid<A::State>,
        instance: Arc<Instance>,
    ) -> Self {
        let vk = {
            // Select a queue family from the physical device
            let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
            let comp_queue_family = physical
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
                [(comp_queue_family, 0.5)].iter().cloned(),
            )
            .unwrap();
            let comp_queue = queues.next().unwrap();

            // Bind the automaton to the device
            automaton.bind_device(&device);

            // Create buffers
            let size = {
                let dim = grid.dim();
                dim.nb_rows * dim.nb_cols
            };
            let src_buf = DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::all(),
                physical.queue_families(),
            )
            .unwrap();
            let dst_buf = DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::all(),
                physical.queue_families(),
            )
            .unwrap();

            VKResources {
                device,
                comp_queue,
                src_buf,
                dst_buf,
            }
        };

        Self {
            name: String::from(name),
            automaton,
            grid: grid.clone(),
            current_gen: 0,
            vk,
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
        for _i in 0..nb_gens {
            // Transform grid into raw data
            let raw_data = self.grid_to_raw();

            // Create CPU accessible buffer that contains the raw data
            let cpu_buffer: Arc<CpuAccessibleBuffer<[u32]>> = CpuAccessibleBuffer::from_iter(
                self.vk.device.clone(),
                BufferUsage::all(),
                true,
                raw_data.into_iter(),
            )
            .unwrap();

            // Descriptor set
            let layout = self.automaton.gpu_layout();
            let set = Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(self.vk.src_buf.clone())
                    .unwrap()
                    .add_buffer(self.vk.dst_buf.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            // Command buffer
            let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
                self.vk.device.clone(),
                self.vk.comp_queue.family(),
            )
            .unwrap()
            .copy_buffer(cpu_buffer.clone(), self.vk.src_buf.clone())
            .unwrap();
            let command_buffer = self
                .automaton
                .gpu_dispatch([self.grid.dim().nb_cols as u32, self.grid.dim().nb_rows as u32, 1], command_buffer, set, self.grid.dim())
                .copy_buffer(self.vk.dst_buf.clone(), cpu_buffer.clone())
                .unwrap()
                .build()
                .unwrap();

            // Execute the command buffer
            let future = sync::now(self.vk.device.clone())
                .then_execute(self.vk.comp_queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap();
            future.wait(None).unwrap();

            // Update grid
            self.grid.switch_data(self.raw_to_grid(cpu_buffer));
        }

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

struct VKResources {
    device: Arc<Device>,
    comp_queue: Arc<Queue>,
    src_buf: Arc<DeviceLocalBuffer<[u32]>>,
    dst_buf: Arc<DeviceLocalBuffer<[u32]>>,
}
