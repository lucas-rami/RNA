// Standard library
use std::thread;

// CELL
use super::{
    universe_history::{HistoryRequest, HistoryResponse, UniverseHistory},
    Simulator,
};
use crate::{
    advanced_channels::{
        oneway_channel, twoway_channel, MasterEndpoint, SimpleSender, TransmittingEnd,
    },
    automaton::{CPUCell, GPUCell},
    universe::{CPUUniverse, GPUUniverse, Universe},
};

pub struct AsyncSimulator<U: Universe> {
    runner_comm: SimpleSender<usize>,
    history_comm: MasterEndpoint<HistoryRequest<U>, HistoryResponse<U>>,
    max_gen: usize,
}

impl<U: Universe> AsyncSimulator<U> {
    fn get_generation_blocking(&self, gen: usize, blocking: bool) -> Option<U> {
        match self
            .history_comm
            .send_and_wait_for_response(HistoryRequest::GetGen(gen, blocking))
        {
            HistoryResponse::GetGen(opt_universe) => opt_universe,
            _ => panic!(ERR_INCORRECT_RESPONSE),
        }
    }

    fn get_difference_blocking(
        &self,
        ref_gen: usize,
        target_gen: usize,
        blocking: bool,
    ) -> Option<<U as Universe>::Diff> {
        match self
            .history_comm
            .send_and_wait_for_response(HistoryRequest::GetDiff(ref_gen, target_gen, blocking))
        {
            HistoryResponse::GetDiff(opt_diff) => opt_diff,
            _ => panic!(ERR_INCORRECT_RESPONSE),
        }
    }
}

impl<U: Universe> Simulator for AsyncSimulator<U> {
    type U = U;

    fn run(&mut self, nb_gens: usize) {
        self.runner_comm.send(nb_gens);
        self.max_gen += nb_gens;
    }

    fn get_highest_generation(&self) -> usize {
        self.max_gen
    }

    fn get_generation(&self, gen: usize) -> Option<Self::U> {
        if gen <= self.max_gen {
            self.get_generation_blocking(gen, true)
        } else {
            None
        }
    }

    fn get_difference(
        &self,
        ref_gen: usize,
        target_gen: usize,
    ) -> Option<<Self::U as Universe>::Diff> {
        if target_gen < self.max_gen {
            None
        } else {
            self.get_difference_blocking(ref_gen, target_gen, true)
        }
    }
}

impl<U: CPUUniverse> AsyncSimulator<U>
where
    U::Cell: CPUCell,
{
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
                        current_universe =
                            U::cpu_evolve_callback(current_universe, nb_gens, callback)
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

impl<U: GPUUniverse> AsyncSimulator<U>
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
