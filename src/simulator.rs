// Local
mod async_simulator;
mod sync_simulator;
mod universe_history;
use crate::universe::Universe;
pub use async_simulator::AsyncSimulator;
pub use sync_simulator::SyncSimulator;
use universe_history::UniverseHistory;

pub trait Simulator {
    type Universe: Universe;

    fn run(&mut self, n_gens: usize);

    fn get_highest_generation(&self) -> usize;

    fn get_generation(&self, gen: usize) -> Option<Self::Universe>;

    fn goto(&mut self, target_gen: usize) {
        let max_gen = self.get_highest_generation();
        if target_gen > max_gen {
            self.run(target_gen - max_gen);
        }
    }
}
