// Standard library
use std::hash::Hash;
use std::sync::{mpsc, Arc};
use std::thread;

// External libraries
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
mod compute;
use crate::grid::{Dimensions, Grid, GridHistory, GridView, Position};
use compute::ComputeCluster;

// ############# Traits and associated structs #############

pub trait CellularAutomaton {
    type State: Copy + Default + Eq + PartialEq;

    fn name(&self) -> &str {
        "Cellular Automaton"
    }
}

pub trait CPUComputableAutomaton: CellularAutomaton {
    fn update_cpu<'a>(&self, grid: &GridView<'a, Self::State>) -> Self::State;
}

pub trait GPUComputableAutomaton: CellularAutomaton
where
    Self::State: Transcoder,
{
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;
    type PushConstants: Copy;

    fn vk_setup(&self, device: &Arc<Device>) -> PipelineInfo<Self::Pipeline>;
    fn push_constants(&self, grid: &Grid<Self::State>) -> Self::PushConstants;
}

pub trait Transcoder {
    fn encode(&self) -> u32;
    fn decode(id: u32) -> Self;
}

#[derive(Clone)]
pub struct PipelineInfo<P>
where
    P: ComputePipelineAbstract + Send + Sync + 'static,
{
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<P>,
}

// ############# Simulator #############

pub struct Simulator<A: CellularAutomaton> {
    name: String,
    automaton: A,
    grid: Grid<A::State>,
    current_gen: u64,
    use_gpu: bool,
}

impl<A: CellularAutomaton> Simulator<A> {
    pub fn name(&self) -> &str {
        &self.name[..]
    }

    pub fn automaton(&self) -> &A {
        &self.automaton
    }

    pub fn run(&mut self, nb_gens: u64) {
        if self.use_gpu {
            self.run_gpu(nb_gens)
        } else {
            self.run_cpu(nb_gens)
        }
    }

    fn run_cpu(&mut self, nb_gens: u64) {
        // for _ in 0..nb_gens {
        //     let dim = self.grid.dim();
        //     let mut new_data = Vec::with_capacity(dim.size() as usize);
        //     for row in 0..dim.height() {
        //         for col in 0..dim.width() {
        //             let pos = Position::new(col, row);
        //             let view = self.grid.view(pos.clone());
        //             let new_state = self.automaton.update_cpu(&view);
        //             new_data.push(new_state);
        //         }
        //     }
        //     self.grid.switch_data(new_data);
        // }
    }

    fn run_gpu(&mut self, nb_gens: u64) {}
}

impl<A: CPUComputableAutomaton> Simulator<A> {
    pub fn new(name: &str, automaton: A, grid: Grid<A::State>) -> Self {
        Self {
            name: String::from(name),
            automaton,
            grid: grid.clone(),
            current_gen: 0,
            use_gpu: false,
        }
    }
}

// pub struct GPUSimulator<A: GPUComputableAutomaton>
// where
//     A::State: PartialEq + Eq + Hash,
// {
//     name: String,
//     automaton: A,
//     grid: Vec<Grid<A::State>>,
//     tx_op: mpsc::Sender<ComputeOP>,
//     rx_data: mpsc::Receiver<Vec<Arc<CpuAccessibleBuffer<[u32]>>>>,
// }

// impl<A: GPUComputableAutomaton> GPUSimulator<A>
// where
//     A::State: PartialEq + Eq + Hash,
// {
//     pub fn new(name: &str, automaton: A, grid: Grid<A::State>, instance: Arc<Instance>) -> Self {
//         // Create cluster
//         let cluster = {
//             // Select a queue family from the physical device
//             let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
//             let comp_q_family = physical
//                 .queue_families()
//                 .find(|&q| q.supports_compute())
//                 .unwrap();

//             // Create a logical device and retreive the compute queue handle
//             let (device, mut queues) = Device::new(
//                 physical,
//                 physical.supported_features(),
//                 &DeviceExtensions {
//                     khr_storage_buffer_storage_class: true,
//                     ..DeviceExtensions::none()
//                 },
//                 [(comp_q_family, 0.5)].iter().cloned(),
//             )
//             .unwrap();
//             let queue = queues.next().unwrap();

//             // Get pipeline information from automaton and create compute manager
//             let pipe_info = automaton.vk_setup(&device);
//             let pc = automaton.push_constants(&grid);
//             ComputeCluster::new(device, queue, pipe_info, pc, 4, grid.dim())
//         };

//         // Create channels to communicate with compute cluster and launch it in a new thread
//         let (tx_op, rx_op) = mpsc::channel();
//         let (tx_data, rx_data) = mpsc::channel();
//         thread::spawn(move || cluster.dispatch(rx_op, tx_data));

//         // Create simulator
//         let sim = Self {
//             name: String::from(name),
//             automaton,
//             grid: vec![grid],
//             tx_op,
//             rx_data,
//         };

//         // Initialize the compute cluster and return simulator
//         sim.tx_op
//             .send(ComputeOP::Reset(sim.grid_to_raw(0)))
//             .expect(ERR_DEAD_CLUSTER);
//         sim
//     }

//     fn run(&mut self, nb_gens: u64) -> () {
//         self.tx_op
//             .send(ComputeOP::Run(nb_gens))
//             .expect(ERR_DEAD_CLUSTER);

//         for i in 0..nb_gens {
//             let cpu_bufs = self.rx_data.recv().expect(ERR_DEAD_CLUSTER);
//             for buf in cpu_bufs {
//                 self.grid.push(self.raw_to_grid(buf));
//             }
//         }
//     }
// }

pub enum ComputeOP {
    Run(u64),
    Reset(Vec<u32>),
}

// const ERR_DEAD_CLUSTER: &str = "The compute cluster died.";
