// Standard library
use std::sync::Arc;
use std::thread;

// External libraries
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice};

// CELL
use super::compute::{CPUCompute, GPUCompute};
use crate::advanced_channels::{self, MasterEndpoint, SimpleSender};
use crate::automaton::{Cell, CellularAutomaton, UpdateCPU, UpdateGPU};
use crate::grid::{Grid, GridHistory, GridHistoryOP};

pub struct Simulator<C: Cell> {
    automaton: CellularAutomaton<C>,
    max_gen: usize,
    grid_manager: MasterEndpoint<GridHistoryOP<C>, Option<Grid<C>>>,
    compute_manager: SimpleSender<ComputeOP<C>>,
}

impl<C: Cell> Simulator<C> {
    pub fn automaton(&self) -> &CellularAutomaton<C> {
        &self.automaton
    }

    pub fn highest_gen(&self) -> usize {
        self.max_gen
    }

    pub fn run(&mut self, nb_gens: usize) {
        self.compute_manager.send(ComputeOP::Run(nb_gens));
        self.max_gen += nb_gens;
    }

    pub fn goto(&mut self, target_gen: usize) {
        if target_gen > self.max_gen {
            self.run(target_gen - self.max_gen);
        }
    }

    pub fn get_gen(&mut self, gen: usize, run_to: bool) -> Option<Grid<C>> {
        if self.max_gen < gen {
            if run_to {
                self.run(gen - self.max_gen);
            } else {
                return None;
            }
        }

        self.grid_manager
            .send_and_wait_for_response(GridHistoryOP::GetGen {
                gen,
                blocking: true,
            })
    }
}

impl<C: UpdateCPU> Simulator<C> {
    pub fn new_cpu_sim(automaton: CellularAutomaton<C>, grid: &Grid<C>) -> Self {
        // Create communication channels
        let (grid_master, grid_slave) = advanced_channels::twoway_channel();
        let (compute_sender, compute_receiver) = advanced_channels::oneway_channel();

        // Dispatch a CPUCompute thread and GridHistory thread
        let compute = CPUCompute::new(grid.clone());
        let history = GridHistory::new(&grid, 10);
        let grid_third_party = grid_master.create_third_party();
        thread::spawn(move || compute.dispatch(compute_receiver, grid_third_party));
        thread::spawn(move || history.dispatch(grid_slave));

        // Send a Reset signal to the compute thread to initialize the grid
        compute_sender.send(ComputeOP::Reset(grid.clone()));

        // Create the simulator
        Self {
            automaton,
            max_gen: 0,
            grid_manager: grid_master,
            compute_manager: compute_sender,
        }
    }
}

impl<C: UpdateGPU> Simulator<C> {
    pub fn new_gpu_sim(
        automaton: CellularAutomaton<C>,
        grid: &Grid<C>,
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
        let (grid_master, grid_slave) = advanced_channels::twoway_channel();
        let (compute_sender, compute_receiver) = advanced_channels::oneway_channel();

        // Dispatch a GPUCompute thread and GridHistory thread
        let history = GridHistory::new(&grid, 10);
        let grid_third_party = grid_master.create_third_party();
        thread::spawn(move || compute.dispatch(compute_receiver, grid_third_party));
        thread::spawn(move || history.dispatch(grid_slave));

        // Send a Reset signal to the compute thread to initialize the grid
        compute_sender.send(ComputeOP::Reset(grid.clone()));

        // Create the simulator
        Self {
            automaton,
            max_gen: 0,
            grid_manager: grid_master,
            compute_manager: compute_sender,
        }
    }
}

pub enum ComputeOP<C: Cell> {
    Reset(Grid<C>),
    Run(usize),
}
