// CELL
use crate::automaton::CellularAutomaton;
use crate::universe::{Universe, UniverseManager};

/// Simulator using CPU to compute new generations

pub struct Simulator<U: Universe, M: UniverseManager<U = U>> {
    automaton: CellularAutomaton<U::Cell>,
    universe_manager: M,
    max_gen: usize,
}

impl<U: Universe, M: UniverseManager<U = U>> Simulator<U, M> {
    pub fn new(automaton: CellularAutomaton<U::Cell>, start_universe: U, mut manager: M) -> Self {
        manager.reset(&start_universe);
        Self {
            automaton,
            universe_manager: manager,
            max_gen: 0,
        }
    }

    #[inline]
    fn automaton(&self) -> &CellularAutomaton<U::Cell> {
        &self.automaton
    }

    #[inline]
    fn universe_manager(&self) -> &M {
        &self.universe_manager
    }

    #[inline]
    fn highest_gen(&self) -> usize {
        self.max_gen
    }

    #[inline]
    fn generation(&self, gen: usize) -> Option<U> {
        self.universe_manager.generation(gen)
    }

    #[inline]
    fn difference(&self, ref_gen: usize, target_gen: usize) -> Option<U::Diff> {
        if target_gen < ref_gen {
            panic!(ERR_INVALID_DIFFERENCE)
        }
        self.universe_manager.difference(ref_gen, target_gen)
    }

    fn run(&mut self, nb_gens: usize) {
        self.universe_manager.run(nb_gens);
        self.max_gen += nb_gens;
    }

    fn goto(&mut self, target_gen: usize) {
        let max_gen = self.highest_gen();
        if target_gen > max_gen {
            self.run(target_gen - max_gen);
        }
    }

    fn generation_run_to(&mut self, gen: usize) -> U {
        if self.max_gen < gen {
            self.run(gen - self.max_gen);
        }
        self.universe_manager
            .generation(gen)
            .expect(ERR_EXPECTED_UNIVERSE)
    }

    fn difference_run_to(&mut self, ref_gen: usize, target_gen: usize) -> U::Diff {
        if target_gen < ref_gen {
            panic!(ERR_INVALID_DIFFERENCE)
        }

        if self.max_gen < target_gen {
            self.run(target_gen - self.max_gen);
        }
        self.universe_manager
            .difference(ref_gen, target_gen)
            .expect(ERR_EXPECTED_DIFFERENCE)
    }
}

const ERR_EXPECTED_UNIVERSE: &str = "Expected to receive an actual universe.";
const ERR_EXPECTED_DIFFERENCE: &str = "Expected to receive an actual difference.";
const ERR_INVALID_DIFFERENCE: &str =
    "Reference generation must be smaller or equal to target generation.";
