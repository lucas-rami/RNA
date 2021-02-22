// CELL
mod async_simulator;
mod sync_simulator;
mod universe_history;
use crate::universe::Universe;
pub use async_simulator::AsyncSimulator;
pub use sync_simulator::SyncSimulator;
use universe_history::UniverseHistory;

pub trait Simulator {
    type Universe: Universe;

    fn run(&mut self, nb_gens: usize);

    fn get_highest_generation(&self) -> usize;

    fn get_generation(&self, gen: usize) -> Option<Self::Universe>;

    fn goto(&mut self, target_gen: usize) {
        let max_gen = self.get_highest_generation();
        if target_gen > max_gen {
            self.run(target_gen - max_gen);
        }
    }

    fn run_to_generation(&mut self, target_gen: usize) -> Self::Universe {
        let max_gen = self.get_highest_generation();
        if max_gen < target_gen {
            self.run(target_gen - max_gen);
        }
        self.get_generation(target_gen)
            .expect(ERR_EXPECTED_UNIVERSE)
    }
}

const ERR_EXPECTED_UNIVERSE: &str = "Expected to receive an actual universe.";
