// Standard library
use std::thread;

// CELL
use super::{CPUUniverse, GPUUniverse, Simulator, Universe, UniverseDiff};
use crate::advanced_channels::{
    oneway_channel, twoway_channel, MailType, MasterEndpoint, SimpleReceiver, SimpleSender,
    SlaveEndpoint, ThirdPartySender, TransmittingEnd,
};
use crate::automaton::{CPUCell, GPUCell};

/// SyncSimulator

pub struct SyncSimulator<U: Universe> {
    current_gen: U,
    history: UniverseHistory<U>,
    evolve_fn: fn(U) -> U,
    max_gen: usize,
}

impl<U: Universe> SyncSimulator<U> {
    fn new(start_universe: U, f_check: usize, evolve_fn: fn(U) -> U) -> Self {
        Self {
            current_gen: start_universe.clone(),
            history: UniverseHistory::new(start_universe, f_check),
            evolve_fn,
            max_gen: 0,
        }
    }
}

impl<U: Universe> Simulator for SyncSimulator<U> {
    type U = U;

    fn run(&mut self, nb_gens: usize) {
        let mut universe = self.current_gen.clone();
        let evolve_once = self.evolve_fn;
        for _ in 0..nb_gens {
            universe = evolve_once(universe);
            self.history.push(universe.clone());
        }
        self.current_gen = universe;
        self.max_gen += nb_gens;
    }

    fn reset(&mut self, start_universe: &Self::U) {
        self.current_gen = start_universe.clone();
    }

    fn get_highest_generation(&self) -> usize {
        self.max_gen
    }

    fn get_generation(&self, gen: usize) -> Option<Self::U> {
        self.history.get_gen(gen)
    }

    fn get_difference(
        &self,
        ref_gen: usize,
        target_gen: usize,
    ) -> Option<<Self::U as Universe>::Diff> {
        self.history.get_diff(ref_gen, target_gen)
    }
}

impl<U: CPUUniverse> SyncSimulator<U>
where
    U::Cell: CPUCell,
{
    pub fn cpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::evolve_once)
    }
}

impl<U: GPUUniverse> SyncSimulator<U>
where
    U::Cell: GPUCell,
{
    pub fn gpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::evolve_once)
    }
}

/// AsyncSimulator

pub struct AsyncSimulator<U: Universe> {
    runner_comm: SimpleSender<RunnerOP<U>>,
    history_comm: MasterEndpoint<HistoryRequest<U>, HistoryResponse<U>>,
    max_gen: usize,
}

impl<U: Universe> AsyncSimulator<U> {
    pub fn new(start_universe: U, f_check: usize, evolve_fn: fn(U) -> U) -> Self {
        // Create communication channels
        let (runner_op_sender, runner_op_receiver) = oneway_channel();
        let (history_master, history_slave) = twoway_channel();
        let history_data_sender = history_master.create_third_party();

        // Start 2 detached threads
        UniverseHistory::new(start_universe.clone(), f_check).detach(history_slave);
        Self::universe_runner(
            start_universe,
            runner_op_receiver,
            history_data_sender,
            evolve_fn,
        );

        Self {
            runner_comm: runner_op_sender,
            history_comm: history_master,
            max_gen: 0,
        }
    }

    fn universe_runner(
        start_universe: U,
        op_recv: SimpleReceiver<RunnerOP<U>>,
        history_tx: ThirdPartySender<HistoryRequest<U>>,
        evolve_fn: fn(U) -> U,
    ) {
        thread::spawn(move || {
            let mut current_universe = start_universe;
            loop {
                match op_recv.wait_for_mail() {
                    Ok(op) => match op {
                        RunnerOP::Reset(universe) => current_universe = universe,
                        RunnerOP::Run(nb_gens) => {
                            current_universe = Self::evolve_mailbox(
                                current_universe,
                                nb_gens,
                                &history_tx,
                                evolve_fn,
                            )
                        }
                    },
                    Err(_) => break, // Manager died, time to die
                }
            }
        });
    }

    fn evolve_mailbox(
        start_universe: U,
        nb_gens: usize,
        mailbox: &ThirdPartySender<HistoryRequest<U>>,
        evolve_once: impl Fn(U) -> U,
    ) -> U {
        let mut universe = start_universe;
        for _ in 0..nb_gens {
            universe = evolve_once(universe);
            mailbox.send(HistoryRequest::Push(universe.clone()));
        }
        universe
    }

    fn get_generation_blocking(&self, gen: usize, blocking: bool) -> Option<U> {
        match self
            .history_comm
            .send_and_wait_for_response(HistoryRequest::GetGen(gen, blocking))
        {
            HistoryResponse::GetGen(opt_universe) => opt_universe,
            _ => panic!(ERR_INCORRECT_RESPONSE),
        }
    }

    fn difference_blocking(
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
        self.runner_comm.send(RunnerOP::Run(nb_gens));
        self.max_gen += nb_gens;
    }

    fn reset(&mut self, start_universe: &Self::U) {
        self.runner_comm
            .send(RunnerOP::Reset(start_universe.clone()));
        self.max_gen = 0;
    }

    fn get_highest_generation(&self) -> usize {
        self.max_gen
    }

    fn get_generation(&self, gen: usize) -> Option<Self::U> {
        if gen < self.max_gen {
            None
        } else {
            self.get_generation_blocking(gen, true)
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
            self.difference_blocking(ref_gen, target_gen, true)
        }
    }
}

impl<U: CPUUniverse> AsyncSimulator<U>
where
    U::Cell: CPUCell,
{
    pub fn cpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::evolve_once)
    }
}

impl<U: GPUUniverse> AsyncSimulator<U>
where
    U::Cell: GPUCell,
{
    pub fn gpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::evolve_once)
    }
}

pub enum RunnerOP<U: Universe> {
    Run(usize),
    Reset(U),
}

/// UniverseHistory

pub struct UniverseHistory<U: Universe> {
    diffs: Vec<U::Diff>,
    checkpoints: Vec<U>,
    f_check: usize,
    last: U,
}

impl<U: Universe> UniverseHistory<U> {
    pub fn new(start_universe: U, f_check: usize) -> Self {
        Self {
            diffs: vec![],
            checkpoints: vec![start_universe.clone()],
            f_check,
            last: start_universe,
        }
    }

    pub fn push(&mut self, universe: U) {
        let diff = self.last.diff(&universe);
        self.diffs.push(diff);
        if self.f_check != 0 && self.diffs.len() % self.f_check == 0 {
            self.checkpoints.push(universe.clone());
        }
        self.last = universe;
    }

    pub fn get_gen(&self, gen: usize) -> Option<U> {
        if self.diffs.len() < gen {
            // We don't have that generation
            None
        } else {
            // We have the generation
            if self.f_check != 0 {
                let idx = gen / self.f_check;
                let shift = gen % self.f_check;

                // Accumulate differences between reference grid and target generation
                let stacked_diffs = U::Diff::stack_mul(&self.diffs[(gen - shift)..gen]);
                Some(
                    self.checkpoints[idx as usize]
                        .clone()
                        .apply_diff(&stacked_diffs),
                )
            } else {
                // Accumulate differences between initial grid and target generation
                let stacked_diffs = U::Diff::stack_mul(&self.diffs[0..gen]);
                Some(self.checkpoints[0].clone().apply_diff(&stacked_diffs))
            }
        }
    }

    pub fn get_diff(&self, ref_gen: usize, target_gen: usize) -> Option<U::Diff> {
        if target_gen < ref_gen {
            panic!("Base generation should be smaller than target generation.");
        }
        if self.diffs.len() < target_gen {
            None
        } else {
            Some(U::Diff::stack_mul(&self.diffs[ref_gen..target_gen]))
        }
    }

    pub fn detach(mut self, endpoint: SlaveEndpoint<HistoryResponse<U>, HistoryRequest<U>>) {
        thread::spawn(move || loop {
            match endpoint.wait_for_mail() {
                MailType::Message(msg, None) => match msg {
                    HistoryRequest::Push(grid) => self.push(grid),
                    _ => panic!(ERR_INCOMPATIBLE_MAIL_TYPE),
                },
                MailType::Message(msg, Some(req)) => match msg {
                    HistoryRequest::GetGen(gen, blocking) => match self.get_gen(gen) {
                        Some(grid) => {
                            req.respond(HistoryResponse::GetGen(Some(grid)));
                        }
                        None => {
                            if blocking {
                                loop {
                                    if let HistoryRequest::Push(grid) = endpoint.wait_for_msg() {
                                        self.push(grid);
                                        if let Some(response_grid) = self.get_gen(gen) {
                                            req.respond(HistoryResponse::GetGen(Some(
                                                response_grid,
                                            )));
                                            break;
                                        }
                                    } else {
                                        panic!(ERR_INCOMPATIBLE_MAIL_TYPE);
                                    }
                                }
                            } else {
                                req.respond(HistoryResponse::GetGen(None));
                            }
                        }
                    },
                    HistoryRequest::GetDiff(ref_gen, target_gen, blocking) => {
                        match self.get_diff(ref_gen, target_gen) {
                            Some(diff) => {
                                req.respond(HistoryResponse::GetDiff(Some(diff)));
                            }
                            None => {
                                if blocking {
                                    loop {
                                        if let HistoryRequest::Push(grid) = endpoint.wait_for_msg()
                                        {
                                            self.push(grid);
                                            if let Some(response_diff) =
                                                self.get_diff(ref_gen, target_gen)
                                            {
                                                req.respond(HistoryResponse::GetDiff(Some(
                                                    response_diff,
                                                )));
                                                break;
                                            }
                                        } else {
                                            panic!(ERR_INCOMPATIBLE_MAIL_TYPE);
                                        }
                                    }
                                } else {
                                    req.respond(HistoryResponse::GetGen(None));
                                }
                            }
                        }
                    }
                    _ => panic!(ERR_INCOMPATIBLE_MAIL_TYPE),
                },
                MailType::DeadChannel => break,
            }
        });
    }
}

pub enum HistoryRequest<U: Universe> {
    Push(U),
    GetDiff(usize, usize, bool),
    GetGen(usize, bool),
}

pub enum HistoryResponse<U: Universe> {
    GetDiff(Option<U::Diff>),
    GetGen(Option<U>),
}

const ERR_INCOMPATIBLE_MAIL_TYPE: &str =
    "The received HistoryRequest is incompatible with the MailType it's included in.";
const ERR_INCORRECT_RESPONSE: &str = "The received response is incompatible with the sent request.";
