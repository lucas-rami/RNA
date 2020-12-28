// Standard library
use std::sync::Arc;

// External libraries
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
pub mod grid2d;
use crate::automaton::{AutomatonCell, CPUCell, GPUCell};

/// Universe

pub trait Universe: Clone + Sized + Send + 'static {
    type Cell: AutomatonCell;
    type Coordinates;
    type Diff: UniverseDiff;

    fn get(&self, coords: Self::Coordinates) -> &Self::Cell;

    fn set(&mut self, coords: Self::Coordinates, val: Self::Cell);

    fn neighbor(
        &self,
        coords: &Self::Coordinates,
        nbor: &<Self::Cell as AutomatonCell>::Neighbor,
    ) -> &Self::Cell;

    fn diff(&self, other: &Self) -> Self::Diff;

    fn apply_diff(self, diff: &Self::Diff) -> Self;
}

pub trait CPUUniverse: Universe
where
    Self::Cell: CPUCell,
{
    fn cpu_evolve(self, nb_gens: usize) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.cpu_evolve_once();
        }
        universe
    }

    fn cpu_evolve_once(self) -> Self {
        self.cpu_evolve(1)
    }

    fn cpu_evolve_callback(self, nb_gens: usize, callback: impl Fn(&Self) -> ()) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.cpu_evolve_once();
            callback(&universe);
        }
        universe
    }
}

pub trait GPUUniverse: Universe
where
    Self::Cell: GPUCell,
{
    fn gpu_evolve(self, nb_gens: usize) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.gpu_evolve_once();
        }
        universe
    }

    fn gpu_evolve_once(self) -> Self {
        self.gpu_evolve(1)
    }

    fn gpu_evolve_callback(self, nb_gens: usize, callback: impl Fn(&Self)) -> Self {
        let mut universe = self;
        for _ in 0..nb_gens {
            universe = universe.gpu_evolve_once();
            callback(&universe);
        }
        universe
    }
}

#[derive(Clone)]
pub struct ShaderInfo {
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<Box<dyn ComputePipelineAbstract + Send + Sync + 'static>>,
}

pub trait UniverseAutomatonShader<C: AutomatonCell>: Universe<Cell = C> {
    fn shader_info(device: &Arc<Device>) -> ShaderInfo;
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
