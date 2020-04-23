// Standard library
use std::marker::PhantomData;
use std::sync::Arc;

// External libraries
use vulkano::buffer::{BufferUsage, DeviceLocalBuffer};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};

// CELL
pub mod automaton;
pub mod grid;
use automaton::{CellularAutomaton, GPUCompute};
use grid::{Dimensions, Grid, Position};

pub trait Simulator<S: Copy, C: CellularAutomaton<S>> {
    fn run(&mut self, nb_gens: u64) -> ();
    fn automaton(&self) -> &C;
    fn cell(&self, pos: &Position) -> &S;
    fn size(&self) -> &Dimensions;
    fn name(&self) -> &str;
    fn current_gen(&self) -> u64;
}

pub struct CPUSimulator<S: Copy, C: CellularAutomaton<S>> {
    name: String,
    automaton: C,
    grid: Grid<S>,
    current_gen: u64,
}

impl<S: Copy, C: CellularAutomaton<S>> CPUSimulator<S, C> {
    pub fn new(name: &str, automaton: C, grid: &Grid<S>) -> Self {
        Self {
            name: String::from(name),
            automaton,
            grid: grid.clone(),
            current_gen: 0,
        }
    }
}

impl<S: Copy, C: CellularAutomaton<S>> Simulator<S, C> for CPUSimulator<S, C> {
    fn run(&mut self, nb_gens: u64) -> () {
        for _ in 0..nb_gens {
            let dim = self.grid.dim();
            let mut new_grid = Grid::new(dim.clone(), &self.automaton.default());
            for row in 0..dim.nb_rows {
                for col in 0..dim.nb_cols {
                    let pos = Position::new(col, row);
                    let view = self.grid.view(pos.clone());
                    let new_state = self.automaton.update_cpu(&view);
                    new_grid.set(&pos, new_state);
                }
            }
            self.grid = new_grid;
        }
        self.current_gen += nb_gens
    }

    fn automaton(&self) -> &C {
        &self.automaton
    }

    fn cell(&self, pos: &Position) -> &S {
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
    instance: Arc<Instance>,
    device: Arc<Device>,
    comp_queue: Arc<Queue>,
    src_buf: Arc<DeviceLocalBuffer<[u8]>>,
    dst_buf: Arc<DeviceLocalBuffer<[u8]>>,
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
