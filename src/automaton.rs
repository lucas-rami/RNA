// Standard library
use std::fmt::Debug;
use std::sync::Arc;

// External libraries
use crossterm::style::StyledContent;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
use crate::universe::CPUUniverse;

pub trait AutomatonCell: Copy + Debug + Default + Eq + PartialEq + Send + Sync + 'static {
    type Neighbor;
    type Encoded: Copy + Send + Sync;

    fn encode(&self) -> Self::Encoded;
    fn decode(encoded: &Self::Encoded) -> Self;

    fn neighborhood() -> &'static [Self::Neighbor];
}

pub trait CPUCell: AutomatonCell {
    fn update<U: CPUUniverse<Cell = Self, Neighbor = Self::Neighbor>>(
        &self,
        universe: &U,
        pos: &U::Position,
    ) -> Self;
}

pub trait GPUCell: AutomatonCell {}

#[derive(Clone)]
pub struct ShaderInfo {
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<Box<dyn ComputePipelineAbstract + Send + Sync + 'static>>,
}

pub trait TermDrawableAutomaton: AutomatonCell {
    fn style(&self) -> StyledContent<char>;
}
