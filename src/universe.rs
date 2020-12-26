// Standard library
use std::sync::Arc;

// External libraries
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
pub mod grid2d;
pub mod simulator;
use crate::advanced_channels::TransmittingEnd;
use crate::automaton::{AutomatonCell, CPUCell, GPUCell};

/// Universe

pub trait Universe: Clone + Sized + Send + 'static {
    type Cell: AutomatonCell<Neighbor = Self::Neighbor>;
    type Position;
    type Neighbor;
    type Diff: UniverseDiff;

    fn get(&self, pos: Self::Position) -> &Self::Cell;

    fn set(&mut self, pos: Self::Position, val: Self::Cell);

    fn neighbor(&self, pos: &Self::Position, nbor: &Self::Neighbor) -> &Self::Cell;

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
    Self::Cell: GPUCell,
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

pub trait UniverseAutomatonShader<C: AutomatonCell>:
    Universe<Cell = C, Neighbor = C::Neighbor>
{
    fn shader_info(device: &Arc<Device>) -> ShaderInfo;
}

#[derive(Clone)]
pub struct ShaderInfo {
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<Box<dyn ComputePipelineAbstract + Send + Sync + 'static>>,
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

/// Simulator

pub trait Simulator {
    type U: Universe;

    fn run(&mut self, nb_gens: usize);

    fn reset(&mut self, start_universe: &Self::U);

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

const ERR_EXPECTED_UNIVERSE: &str = "Expected to receive an actual universe.";
const ERR_EXPECTED_DIFFERENCE: &str = "Expected to receive an actual difference.";
const ERR_INVALID_DIFFERENCE: &str =
    "Reference generation must be smaller or equal to target generation.";
