// Standard library
use std::sync::Arc;

// External libraries
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::{
    DescriptorSetsCollection, PersistentDescriptorSet, UnsafeDescriptorSetLayout,
};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::ComputePipelineAbstract;
use vulkano::sync::{self, GpuFuture};

// CELL
use super::grid::{Dimensions, Grid, Position};
use super::{CellularAutomaton, Simulator};

pub trait GPUComputableAutomaton: CellularAutomaton {
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;

    fn id_from_state(&self, state: &Self::State) -> u32;
    fn state_from_id(&self, id: u32) -> Self::State;
    fn vk_setup(&mut self, device: &Arc<Device>) -> PipelineInfo<Self::Pipeline>;
}

#[derive(Clone)]
pub struct PipelineInfo<P>
where
    P: ComputePipelineAbstract + Send + Sync + 'static,
{
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<P>,
}

pub struct GPUSimulator<A: GPUComputableAutomaton> {
    name: String,
    automaton: A,
    grid: Grid<A::State>,
    current_gen: u64,
    manager: ComputeManager<A::Pipeline>,
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

            // Get pipeline information from automaton and create compute manager
            let pipe_info = automaton.vk_setup(&device);
            ComputeManager::new(device, queue, pipe_info, 4, grid.dim())
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

struct ComputeManager<P: ComputePipelineAbstract + Send + Sync + 'static> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipe_info: PipelineInfo<P>,
    gpu_bufs: Vec<Arc<DeviceLocalBuffer<[u32]>>>,
    comp_units: Vec<ComputeUnit>,
    next_exec: usize,
    next_copy: usize,
}

impl<P: ComputePipelineAbstract + Send + Sync + 'static> ComputeManager<P> {
    fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipe_info: PipelineInfo<P>,
        nb_comp_units: usize,
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
                Arc::clone(&device),
                Arc::clone(&queue),
                &pipe_info,
                Arc::clone(&gpu_bufs[i]),
                Arc::clone(&gpu_bufs[j]),
                size,
            ))
        }

        Self {
            device,
            queue,
            pipe_info,
            gpu_bufs,
            comp_units,
            next_exec: 0,
            next_copy: 0,
        }
    }
}

struct ComputeUnit {
    device: Arc<Device>,
    queue: Arc<Queue>,
    cpu_out: Arc<CpuAccessibleBuffer<[u32]>>,
    cmd: AutoCommandBuffer,
}

impl ComputeUnit {
    fn new<T>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipe_info: &PipelineInfo<T>,
        gpu_src: Arc<DeviceLocalBuffer<[u32]>>,
        gpu_dst: Arc<DeviceLocalBuffer<[u32]>>,
        size: &Dimensions,
    ) -> Self where T: ComputePipelineAbstract + Send + Sync + 'static, {
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
            PersistentDescriptorSet::start(pipe_info.layout.clone())
                .add_buffer(gpu_src.clone())
                .unwrap()
                .add_buffer(gpu_dst.clone())
                .unwrap()
                .build()
                .unwrap(),
        );
        let cmd = AutoCommandBufferBuilder::primary(device.clone(), queue.family()).unwrap()
            .dispatch([size.nb_cols as u32, size.nb_rows as u32, 1], pipe_info.pipeline.clone(), set, ())
            .unwrap()
            .copy_buffer(gpu_dst.clone(), cpu_out.clone())
            .unwrap()
            .build()
            .unwrap();

        Self {
            device,
            queue,
            cpu_out,
            cmd,
        }
    }

    fn exec(&self) -> () {
        // let future = sync::now(self.device.clone())
        //     .then_execute(self.queue.clone(), submit_cmd)
        //     .unwrap()
        //     .then_signal_fence_and_flush()
        //     .unwrap();
        // future.wait(None).unwrap();
    }
}
