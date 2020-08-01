// Standard library
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

// External libraries
use crossterm::style::StyledContent;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
use crate::universe::{CPUUniverse, GPUUniverse};

pub trait AutomatonCell: Copy + Debug + Default + Eq + PartialEq + Send + Sync + 'static {
    type Neighbor;
    type Encoded: Copy + Send + Sync;

    fn encode(&self) -> Self::Encoded;
    fn decode(encoded: &Self::Encoded) -> Self;

    fn neighborhood() -> &'static [Self::Neighbor];
}

pub struct CellularAutomaton<C: AutomatonCell> {
    name: String,
    _marker: PhantomData<C>,
}

impl<C: AutomatonCell> CellularAutomaton<C> {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name[..]
    }
}

pub trait NeighborhoodView {
    type Cell: AutomatonCell;

    fn get_by_idx(&self, idx: usize) -> &Self::Cell;
    fn get_by_name(&self, name: &str) -> &Self::Cell;
    fn get_all<'a>(&'a self) -> Vec<&'a Self::Cell>;
}

pub trait CPUCell: AutomatonCell {
    fn update<U: CPUUniverse<Cell = Self, Neighbor = Self::Neighbor>>(
        &self,
        universe: &U,
        pos: &U::Position,
    ) -> Self;
}

pub trait GPUCell<U: GPUUniverse<Cell = Self>>: AutomatonCell {
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;
    fn shader_info(device: &Arc<Device>) -> ShaderInfo<Self::Pipeline>;
}

#[derive(Clone)]
pub struct ShaderInfo<P>
where
    P: ComputePipelineAbstract + Send + Sync + 'static,
{
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<P>,
}

pub trait TermDrawableAutomaton: AutomatonCell {
    fn style(&self) -> StyledContent<char>;
}
