// Standard library
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;

// External libraries
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
mod compute;
use crate::grid::{Grid, GridHistory, GridHistoryOP, GridView, PositionIterator};
use compute::{CPUCompute, GPUCompute};

// ############# Traits and associated structs #############

pub trait CellType: Copy + Default + std::fmt::Debug + Eq + PartialEq + Send {}

pub trait CellularAutomaton: 'static {
    type Cell: CellType;

    fn name(&self) -> &str {
        "Cellular Automaton"
    }
}

pub trait CPUComputableAutomaton: CellularAutomaton {
    fn update_cell<'a>(grid: &GridView<'a, Self::Cell>) -> Self::Cell;

    fn update_grid(grid: &Grid<Self::Cell>) -> Grid<Self::Cell> {
        let dim = grid.dim();
        let mut new_data = Vec::with_capacity(dim.size() as usize);
        for pos in PositionIterator::new(*dim) {
            let new_cell = Self::update_cell(&grid.view(pos));
            new_data.push(new_cell);
        }
        Grid::from_data(new_data, *dim)
    }
}

pub trait GPUComputableAutomaton: CellularAutomaton
where
    Self::Cell: Transcoder,
{
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;
    type PushConstants: Copy;

    fn vk_setup(&self, device: &Arc<Device>) -> PipelineInfo<Self::Pipeline>;
    fn push_constants(&self, grid: &Grid<Self::Cell>) -> Self::PushConstants;
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
    max_gen: usize,
    tx_comp_op: Sender<ComputeOP<A>>,
    tx_grid_op: Sender<GridHistoryOP<A::Cell>>,
    rx_data: Receiver<Option<Grid<A::Cell>>>,
}

impl<A: CellularAutomaton> Simulator<A> {
    pub fn name(&self) -> &str {
        &self.name[..]
    }

    pub fn automaton(&self) -> &A {
        &self.automaton
    }

    pub fn highest_gen(&self) -> usize {
        self.max_gen
    }

    pub fn run(&mut self, nb_gens: usize) {
        self.tx_comp_op
            .send(ComputeOP::Run(nb_gens))
            .expect(ERR_DEAD_CPU_COMPUTE);
        self.max_gen += nb_gens;
    }

    pub fn goto(&mut self, target_gen: usize) {
        if target_gen > self.max_gen {
            self.run(target_gen - self.max_gen);
        }
    }

    pub fn get_gen(&mut self, gen: usize, run_to: bool) -> Option<Grid<A::Cell>> {
        if self.max_gen < gen {
            if run_to {
                self.run(gen - self.max_gen);
            } else {
                return None;
            }
        }

        self.tx_grid_op
            .send(GridHistoryOP::GetGen {
                gen,
                blocking: true,
            })
            .expect(ERR_DEAD_GRID_HISTORY);
        self.rx_data.recv().expect(ERR_DEAD_GRID_HISTORY)
    }
}

impl<A: CPUComputableAutomaton> Simulator<A> {
    pub fn new_cpu_sim(name: &str, automaton: A, grid: &Grid<A::Cell>) -> Self {
        // Create communication channels
        let (tx_comp_op, rx_comp_op) = mpsc::channel();
        let (tx_grid_op, rx_grid_op) = mpsc::channel();
        let (tx_data, rx_data) = mpsc::channel();

        // Dispatch a CPUCompute thread and GridHistory thread
        let compute = CPUCompute::new();
        let history = GridHistory::new(&grid, 10);
        let tx_grid_op_compute = tx_grid_op.clone();
        thread::spawn(move || compute.dispatch(rx_comp_op, tx_grid_op_compute));
        thread::spawn(move || history.dispatch(rx_grid_op, tx_data));

        // Send a Reset signal to the compute thread to initialize the grid
        tx_comp_op
            .send(ComputeOP::Reset(grid.clone()))
            .expect(ERR_DEAD_CPU_COMPUTE);

        // Create the simulator
        Self {
            name: String::from(name),
            automaton,
            max_gen: 0,
            tx_comp_op,
            tx_grid_op,
            rx_data,
        }
    }
}

impl<A: GPUComputableAutomaton> Simulator<A>
where
    A::Cell: Transcoder,
{
    pub fn new_gpu_sim(
        name: &str,
        automaton: A,
        grid: &Grid<A::Cell>,
        instance: Arc<Instance>,
    ) -> Self {
        // Create GPUCompute struct
        let compute = {
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
            GPUCompute::new(device, queue, pipe_info, pc, 4, grid.dim())
        };

        // Create communication channels
        let (tx_comp_op, rx_comp_op) = mpsc::channel();
        let (tx_grid_op, rx_grid_op) = mpsc::channel();
        let (tx_data, rx_data) = mpsc::channel();

        // Dispatch a GPUCompute thread and GridHistory thread
        let history = GridHistory::new(&grid, 10);
        let tx_grid_op_compute = tx_grid_op.clone();
        thread::spawn(move || compute.dispatch(rx_comp_op, tx_grid_op_compute));
        thread::spawn(move || history.dispatch(rx_grid_op, tx_data));

        // Send a Reset signal to the compute thread to initialize the grid
        tx_comp_op
            .send(ComputeOP::Reset(grid.clone()))
            .expect(ERR_DEAD_GPU_COMPUTE);

        // Create the simulator
        Self {
            name: String::from(name),
            automaton,
            max_gen: 0,
            tx_comp_op,
            tx_grid_op,
            rx_data,
        }
    }
}

pub enum ComputeOP<A: CellularAutomaton> {
    Reset(Grid<A::Cell>),
    Run(usize),
}

const ERR_DEAD_CPU_COMPUTE: &str = "The CPUCompute thread terminated unexpectedly.";
const ERR_DEAD_GPU_COMPUTE: &str = "The GPUCompute thread terminated unexpectedly.";
const ERR_DEAD_GRID_HISTORY: &str = "The GridHistory thread terminated unexpectedly.";

#[cfg(test)]
mod tests {

    use super::*;
    use crate::game_of_life::*;
    use crate::grid::Grid;

    #[test]
    fn cpu_get_gen() {
        let grid = conway_canon();
        let mut sim = Simulator::new_cpu_sim("Simulator", GameOfLife::new(), &grid);
        sim.run(20);
        assert_eq!(
            sim.get_gen(20, false).unwrap(),
            compute_gen::<GameOfLife>(&grid, 20)
        );
    }

    #[test]
    fn cpu_get_gen_on_demand() {
        let grid = conway_canon();
        let mut sim = Simulator::new_cpu_sim("Simulator", GameOfLife::new(), &grid);
        assert_eq!(
            sim.get_gen(20, true).unwrap(),
            compute_gen::<GameOfLife>(&grid, 20)
        );
    }

    #[test]
    fn cpu_get_multiple_gens() {
        let grid = conway_canon();
        let mut sim = Simulator::new_cpu_sim("Simulator", GameOfLife::new(), &grid);
        sim.run(50);

        let gens = vec![1, 7, 10, 19, 20];

        for gen in gens {
            assert_eq!(
                sim.get_gen(gen, false).unwrap(),
                compute_gen::<GameOfLife>(&grid, gen)
            );
        }
    }

    fn compute_gen<A: CPUComputableAutomaton>(
        base: &Grid<A::Cell>,
        nb_gens: usize,
    ) -> Grid<A::Cell> {
        let mut grid = base.clone();
        for _i in 0..nb_gens {
            grid = A::update_grid(&grid);
        }
        grid
    }
}
