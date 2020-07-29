// CELL
pub mod grid2d;
pub mod manager;
use crate::advanced_channels::TransmittingEnd;
use crate::automaton::{AutomatonCell, CPUCell, GPUCell};

/// Universe

pub trait Universe: Clone + Sized + Send + 'static {
    type Cell: AutomatonCell;
    type Position;
    type Diff: UniverseDiff;

    fn get(&self, pos: Self::Position) -> &Self::Cell;

    fn diff(&self, other: &Self) -> Self::Diff;

    fn apply_diff(self, diff: &Self::Diff) -> Self;
}

pub trait CPUUniverse: Universe
where
    Self::Cell: CPUCell,
{
    fn evolve(self, nb_gens: usize) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.evolve_once();
        }
        universe
    }

    fn evolve_once(self) -> Self {
        self.evolve(1)
    }

    fn evolve_mailbox<T: TransmittingEnd<MSG = Self>>(self, nb_gens: usize, mailbox: &T) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.evolve_once();
            mailbox.send(universe.clone());
        }
        universe
    }
}

pub trait GPUUniverse: Universe
where
    Self::Cell: GPUCell<Self>,
{
    fn evolve(self, nb_gens: usize) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.evolve_once();
        }
        universe
    }

    fn evolve_once(self) -> Self {
        self.evolve(1)
    }

    fn evolve_mailbox<T: TransmittingEnd<MSG = Self>>(self, nb_gens: usize, mailbox: &T) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.evolve_once();
            mailbox.send(universe.clone());
        }
        universe
    }
}

/// UniverseDiff

pub trait UniverseDiff: Clone + Send {
    fn no_diff() -> Self;

    fn stack(&mut self, other: &Self);

    fn stack_mul(diffs: &[Self]) -> Self {
        if diffs.len() == 0 {
            Self::no_diff()
        } else {
            let mut acc_diff = diffs[0].clone();
            for next_diff in &diffs[1..] {
                acc_diff.stack(next_diff);
            }
            acc_diff
        }
    }
}

/// UniverseManager

pub trait UniverseManager {
    type U: Universe;

    fn run(&mut self, nb_gens: usize);

    fn reset(&mut self, start_universe: &Self::U);

    fn generation(&self, gen: usize) -> Option<Self::U>;

    fn difference(&self, ref_gen: usize, target_gen: usize) -> Option<<Self::U as Universe>::Diff>;
}
