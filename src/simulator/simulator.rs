// Standard library
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;

// External libraries
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice};

// CELL
use super::{CPUCompute, GPUCompute};
use crate::automaton::{
    CPUComputableAutomaton, CellularAutomaton, GPUComputableAutomaton, Transcoder,
};
use crate::grid::{Grid, GridHistory, GridHistoryOP};

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
        let compute = CPUCompute::new(grid.clone());
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
            GPUCompute::new(device, queue, 16, &grid)
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
