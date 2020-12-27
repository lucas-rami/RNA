// CELL
use super::{Simulator, UniverseHistory};
use crate::{
    automaton::{CPUCell, GPUCell},
    universe::{CPUUniverse, GPUUniverse, Universe},
};

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
        Self::new(start_universe, f_check, U::cpu_evolve_once)
    }
}

impl<U: GPUUniverse> SyncSimulator<U>
where
    U::Cell: GPUCell,
{
    pub fn gpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::gpu_evolve_once)
    }
}
