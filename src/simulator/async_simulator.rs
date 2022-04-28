// Standard library
use std::thread;

// Local
use super::{
    universe_history::{HistoryRequest, HistoryResponse, UniverseHistory},
    Simulator,
};
use crate::{
    advanced_channels::{
        oneway_channel, twoway_channel, MasterEndpoint, SimpleSender, TransmittingEnd,
    },
    automaton::GPUCell,
    universe::{GPUUniverse, GenerationDifference, Universe},
};

pub struct AsyncSimulator<U: Universe, D: GenerationDifference<Universe = U>> {
    runner_comm: SimpleSender<usize>,
    history_comm: MasterEndpoint<HistoryRequest<U>, HistoryResponse<U, D>>,
    max_gen: usize,
}

impl<U: Universe, D: GenerationDifference<Universe = U>> AsyncSimulator<U, D> {
    fn get_generation_blocking(&self, gen: usize, blocking: bool) -> Option<U> {
        match self
            .history_comm
            .send_and_wait_for_response(HistoryRequest::GetGen(gen, blocking))
        {
            HistoryResponse::GetGen(opt_universe) => opt_universe,
            _ => panic!("{}", ERR_INCORRECT_RESPONSE),
        }
    }

    pub fn cpu_backend(start_universe: U, f_check: usize) -> Self {
        // Create communication channels
        let (runner_op_sender, runner_op_receiver) = oneway_channel();
        let (history_master, history_slave) = twoway_channel();
        let history_data_sender = history_master.create_third_party();

        // Start a thread to manage the universe's history
        UniverseHistory::new(start_universe.clone(), f_check).detach(history_slave);

        // Start a thread to handle run commands
        thread::spawn(move || {
            let mut current_universe = start_universe;
            let callback =
                |universe: &U| history_data_sender.send(HistoryRequest::Push(universe.clone()));
            loop {
                match runner_op_receiver.wait_for_mail() {
                    Ok(nb_gens) => {
                        current_universe = U::evolve_callback(current_universe, nb_gens, callback)
                    }
                    Err(_) => break, // Simulator died, time to die
                }
            }
        });

        Self {
            runner_comm: runner_op_sender,
            history_comm: history_master,
            max_gen: 0,
        }
    }
}

impl<U: Universe, D: GenerationDifference<Universe = U>> Simulator for AsyncSimulator<U, D> {
    type Universe = U;

    fn run(&mut self, nb_gens: usize) {
        self.runner_comm.send(nb_gens);
        self.max_gen += nb_gens;
    }

    fn get_highest_generation(&self) -> usize {
        self.max_gen
    }

    fn get_generation(&self, gen: usize) -> Option<Self::Universe> {
        if gen <= self.max_gen {
            self.get_generation_blocking(gen, true)
        } else {
            None
        }
    }
}

impl<U: GPUUniverse, D: GenerationDifference<Universe = U>> AsyncSimulator<U, D>
where
    U::Cell: GPUCell,
{
    pub fn gpu_backend(start_universe: U, f_check: usize) -> Self {
        // Create communication channels
        let (runner_op_sender, runner_op_receiver) = oneway_channel();
        let (history_master, history_slave) = twoway_channel();
        let history_data_sender = history_master.create_third_party();

        // Start a thread to manage the universe's history
        UniverseHistory::new(start_universe.clone(), f_check).detach(history_slave);

        // Start a thread to handle run commands
        thread::spawn(move || {
            let mut current_universe = start_universe;
            let callback =
                |universe: &U| history_data_sender.send(HistoryRequest::Push(universe.clone()));
            loop {
                match runner_op_receiver.wait_for_mail() {
                    Ok(nb_gens) => {
                        current_universe =
                            U::gpu_evolve_callback(current_universe, nb_gens, callback)
                    }
                    Err(_) => break, // Simulator died, time to die
                }
            }
        });

        Self {
            runner_comm: runner_op_sender,
            history_comm: history_master,
            max_gen: 0,
        }
    }
}

const ERR_INCORRECT_RESPONSE: &str = "The received response is incompatible with the sent request.";
