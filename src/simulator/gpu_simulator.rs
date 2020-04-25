// Standard library
use std::marker::PhantomData;
use std::sync::Arc;

// External libraries
use vulkano::buffer::{BufferUsage, DeviceLocalBuffer};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::ComputePipeline;

// CELL
use super::cpu_simulator::CPUSimulator;
use super::grid::{Grid, Position, Dimensions};
use super::{CellularAutomaton, Simulator};

pub trait GPUCompute<S: Copy, Pl>: CellularAutomaton<S> {
    fn state_name(&self, state: &S) -> &str;

    fn update_gpu(&self, device: Arc<Device>) -> ComputePipeline<Pl>;
}

pub struct GPUSimulator<S: Copy, Pl, C: CellularAutomaton<S> + GPUCompute<S, Pl>> {
    simulator: CPUSimulator<S, C>,
    vk: VKResources,
    _marker: PhantomData<Pl>,
}

impl<S: Copy, Pl, C: CellularAutomaton<S> + GPUCompute<S, Pl>> GPUSimulator<S, Pl, C> {
    pub fn new(name: &str, automaton: C, grid: &Grid<S>, instance: Arc<Instance>) -> Self {
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

            // Create buffers
            let size = {
                let dim = grid.dim();
                dim.nb_rows * dim.nb_cols
            };
            let src_buf = DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::uniform_buffer_transfer_destination(),
                physical.queue_families(),
            )
            .unwrap();
            let dst_buf = DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::uniform_buffer_transfer_destination(),
                physical.queue_families(),
            )
            .unwrap();

            VKResources {
                instance,
                device,
                comp_queue,
                src_buf,
                dst_buf,
            }
        };

        Self {
            simulator: CPUSimulator::new(name, automaton, grid),
            vk,
            _marker: PhantomData,
        }
    }
}

impl<S: Copy, Pl, C: CellularAutomaton<S> + GPUCompute<S, Pl>> Simulator<S, C>
    for GPUSimulator<S, Pl, C>
{
    fn run(&mut self, nb_gens: u64) -> () {}

    fn automaton(&self) -> &C {
        self.simulator.automaton()
    }

    fn cell(&self, pos: &Position) -> &S {
        self.simulator.cell(pos)
    }

    fn size(&self) -> &Dimensions {
        self.simulator.size()
    }

    fn name(&self) -> &str {
        self.simulator.name()
    }

    fn current_gen(&self) -> u64 {
        self.simulator.current_gen()
    }
}

struct VKResources {
    instance: Arc<Instance>,
    device: Arc<Device>,
    comp_queue: Arc<Queue>,
    src_buf: Arc<DeviceLocalBuffer<[u8]>>,
    dst_buf: Arc<DeviceLocalBuffer<[u8]>>,
}
