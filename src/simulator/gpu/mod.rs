// Standard library
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
use super::{CellularAutomaton, Simulator};
use crate::grid::{Dimensions, Grid, Position};
use compute::ComputeCluster;

pub trait GPUComputableAutomaton: CellularAutomaton {
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;
    type PushConstants: Copy;

    fn id_from_state(&self, state: &Self::State) -> u32;
    fn state_from_id(&self, id: u32) -> Self::State;
    fn vk_setup(&self, device: &Arc<Device>) -> PipelineInfo<Self::Pipeline>;
    fn push_constants(&self, grid: &Grid<Self::State>) -> Self::PushConstants;
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
    grid: Vec<Grid<A::State>>,
    tx_op: mpsc::Sender<ComputeOP>,
    rx_data: mpsc::Receiver<Vec<Arc<CpuAccessibleBuffer<[u32]>>>>,
}

impl<A: GPUComputableAutomaton> GPUSimulator<A> {
    pub fn new(name: &str, automaton: A, grid: Grid<A::State>, instance: Arc<Instance>) -> Self {
        // Create cluster
        let cluster = {
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
            let pc = automaton.push_constants(&grid);
            ComputeCluster::new(device, queue, pipe_info, pc, 4, grid.dim())
        };

        // Create channels to communicate with compute cluster and launch it
        let (tx_op, rx_op) = mpsc::channel();
        let (tx_data, rx_data) = mpsc::channel();
        thread::spawn(move || cluster.dispatch(rx_op, tx_data));

        // Create simulator
        let sim = Self {
            name: String::from(name),
            automaton,
            grid: vec![grid],
            tx_op,
            rx_data,
        };

        // Initialize the compute cluster and return simulator
        sim.tx_op
            .send(ComputeOP::Reset(sim.grid_to_raw(0)))
            .expect(ERR_DEAD_CLUSTER);
        sim
    }

    fn grid_to_raw(&self, idx: usize) -> Vec<u32> {
        let dim = self.size();
        let size = dim.size();
        let mut raw_data = Vec::with_capacity(size as usize);
        for state in self.grid[idx].iter() {
            raw_data.push(self.automaton.id_from_state(state));
        }
        raw_data
    }

    fn raw_to_grid(&self, cpu_buf: Arc<CpuAccessibleBuffer<[u32]>>) -> Grid<A::State> {
        let dim = self.size();
        let size = dim.size() as usize;
        let raw_data = cpu_buf.read().unwrap();
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(self.automaton.state_from_id(raw_data[i]));
        }
        Grid::from_data(*dim, data)
    }
}

impl<A: GPUComputableAutomaton> Simulator<A> for GPUSimulator<A> {
    fn run(&mut self, nb_gens: u64) -> () {
        self.tx_op
            .send(ComputeOP::Run(nb_gens))
            .expect(ERR_DEAD_CLUSTER);

        for i in 0..nb_gens {
            let cpu_bufs = self.rx_data.recv().expect(ERR_DEAD_CLUSTER);
            for buf in cpu_bufs {
                self.grid.push(self.raw_to_grid(buf));
            }
        }
    }

    fn automaton(&self) -> &A {
        &self.automaton
    }

    fn cell(&self, pos: Position) -> A::State {
        self.grid[self.grid.len() - 1].get(pos)
    }

    fn size(&self) -> &Dimensions {
        self.grid[0].dim()
    }

    fn name(&self) -> &str {
        &self.name[..]
    }

    fn current_gen(&self) -> u64 {
        self.grid.len() as u64
    }
}

pub enum ComputeOP {
    Run(u64),
    Reset(Vec<u32>),
}

const ERR_DEAD_CLUSTER: &str = "The compute cluster died.";
