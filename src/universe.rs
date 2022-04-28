// Standard library
use std::sync::Arc;

// External libraries
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipelineAbstract;

// Local
pub mod grid2d;
use crate::automaton::{Cell, GPUCell};

pub trait Universe: Clone + Sized + Send + 'static {
    type Cell: Cell;
    type Location: Clone;

    fn get(&self, loc: Self::Location) -> Self::Cell;

    fn set(&mut self, loc: Self::Location, val: Self::Cell);

    fn evolve(self, n_gens: usize) -> Self;
    
    fn evolve_callback(self, n_gens: usize, callback: impl Fn(&Self) -> ()) -> Self {
        let mut universe = self;
        for _ in 0..n_gens {
            universe = universe.evolve(1);
            callback(&universe);
        }
        universe
    }
}

pub trait GPUUniverse: Universe
where
    Self::Cell: GPUCell,
{
    fn gpu_evolve(self, n_gens: usize) -> Self;

    fn gpu_evolve_callback(self, n_gens: usize, callback: impl Fn(&Self)) -> Self {
        let mut universe = self;
        for _ in 0..n_gens {
            universe = universe.gpu_evolve(1);
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
pub trait UniverseAutomatonShader<C: Cell>: Universe<Cell = C> {
    fn shader_info(device: &Arc<Device>) -> ShaderInfo;
}

pub trait GenerationDifference: Clone + Send + 'static {
    type Universe: Universe;

    fn empty_diff() -> Self;

    fn get_diff(base: &Self::Universe, target: &Self::Universe) -> Self;

    fn apply_to(&self, base: Self::Universe) -> Self::Universe;

    fn stack(&mut self, other: &Self);

    fn stack_mul(diffs: &[Self]) -> Self {
        if diffs.len() == 0 {
            Self::empty_diff()
        } else {
            let mut acc_diff = diffs[0].clone();
            for next_diff in &diffs[1..] {
                acc_diff.stack(next_diff);
            }
            acc_diff
        }
    }
}
