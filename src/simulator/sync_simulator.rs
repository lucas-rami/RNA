// Local
use super::{Simulator, UniverseHistory};
use crate::{
    automaton::{CPUCell, GPUCell},
    universe::{CPUUniverse, GPUUniverse, GenerationDifference, Universe},
};

pub struct SyncSimulator<U: Universe, D: GenerationDifference<Universe = U>> {
    current_gen: U,
    history: UniverseHistory<U, D>,
    evolve_fn: fn(U) -> U,
    max_gen: usize,
}

impl<U: Universe, D: GenerationDifference<Universe = U>> SyncSimulator<U, D> {
    fn new(start_universe: U, f_check: usize, evolve_fn: fn(U) -> U) -> Self {
        Self {
            current_gen: start_universe.clone(),
            history: UniverseHistory::new(start_universe, f_check),
            evolve_fn,
            max_gen: 0,
        }
    }
}

impl<U: Universe, D: GenerationDifference<Universe = U>> Simulator for SyncSimulator<U, D> {
    type Universe = U;

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

    fn get_generation(&self, gen: usize) -> Option<Self::Universe> {
        self.history.get_gen(gen)
    }
}

impl<U: CPUUniverse, D: GenerationDifference<Universe = U>> SyncSimulator<U, D>
where
    U::Cell: CPUCell,
{
    pub fn cpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::cpu_evolve_once)
    }
}

impl<U: GPUUniverse, D: GenerationDifference<Universe = U>> SyncSimulator<U, D>
where
    U::Cell: GPUCell,
{
    pub fn gpu_backend(start_universe: U, f_check: usize) -> Self {
        Self::new(start_universe, f_check, U::gpu_evolve_once)
    }
}
