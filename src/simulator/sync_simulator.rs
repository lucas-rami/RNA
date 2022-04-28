// Local
use super::{Simulator, UniverseHistory};
use crate::{
    automaton::GPUCell,
    universe::{GPUUniverse, GenerationDifference, Universe},
};

pub struct SyncSimulator<U: Universe, D: GenerationDifference<Universe = U>> {
    current_gen: U,
    history: UniverseHistory<U, D>,
    evolve_fn: fn(U, usize) -> U,
    max_gen: usize,
}

impl<U: Universe, D: GenerationDifference<Universe = U>> SyncSimulator<U, D> {
    fn new(start_universe: U, f_check: usize, evolve_fn: fn(U, usize) -> U) -> Self {
        Self {
            current_gen: start_universe.clone(),
            history: UniverseHistory::new(start_universe, f_check),
            evolve_fn,
            max_gen: 0,
        }
    }

    pub fn cpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::evolve)
    }
}

impl<U: Universe, D: GenerationDifference<Universe = U>> Simulator for SyncSimulator<U, D> {
    type Universe = U;

    fn run(&mut self, n_gens: usize) {
        let mut universe = self.current_gen.clone();
        let evolve = self.evolve_fn;
        for _ in 0..n_gens {
            universe = evolve(universe, 1);
            self.history.push(universe.clone());
        }
        self.current_gen = universe;
        self.max_gen += n_gens;
    }

    fn get_highest_generation(&self) -> usize {
        self.max_gen
    }

    fn get_generation(&self, gen: usize) -> Option<Self::Universe> {
        self.history.get_gen(gen)
    }
}

impl<U: GPUUniverse, D: GenerationDifference<Universe = U>> SyncSimulator<U, D>
where
    U::Cell: GPUCell,
{
    pub fn gpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::gpu_evolve)
    }
}
