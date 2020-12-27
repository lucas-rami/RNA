// CELL
mod async_simulator;
mod sync_simulator;
mod universe_history;
use crate::universe::Universe;
pub use async_simulator::AsyncSimulator;
pub use sync_simulator::SyncSimulator;
use universe_history::UniverseHistory;

pub trait Simulator {
    type U: Universe;

    fn run(&mut self, nb_gens: usize);

    fn get_highest_generation(&self) -> usize;

    fn get_generation(&self, gen: usize) -> Option<Self::U>;

    fn get_difference(
        &self,
        ref_gen: usize,
        target_gen: usize,
    ) -> Option<<Self::U as Universe>::Diff>;

    fn goto(&mut self, target_gen: usize) {
        let max_gen = self.get_highest_generation();
        if target_gen > max_gen {
            self.run(target_gen - max_gen);
        }
    }

    fn run_to_generation(&mut self, target_gen: usize) -> Self::U {
        let max_gen = self.get_highest_generation();
        if max_gen < target_gen {
            self.run(target_gen - max_gen);
        }
        self.get_generation(target_gen)
            .expect(ERR_EXPECTED_UNIVERSE)
    }

    fn run_to_difference(
        &mut self,
        ref_gen: usize,
        target_gen: usize,
    ) -> <Self::U as Universe>::Diff {
        if target_gen < ref_gen {
            panic!(ERR_INVALID_DIFFERENCE)
        }

        let max_gen = self.get_highest_generation();
        if max_gen < target_gen {
            self.run(target_gen - max_gen);
        }
        self.get_difference(ref_gen, target_gen)
            .expect(ERR_EXPECTED_DIFFERENCE)
    }
}

const ERR_EXPECTED_DIFFERENCE: &str = "Expected to receive an actual difference.";
const ERR_EXPECTED_UNIVERSE: &str = "Expected to receive an actual universe.";
const ERR_INVALID_DIFFERENCE: &str =
    "Reference generation must be smaller or equal to target generation.";
