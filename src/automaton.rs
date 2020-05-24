// Standard library
use std::sync::Arc;

// External libraries
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipelineAbstract;
use crossterm::style::StyledContent;

// CELL
use crate::grid::{Grid, GridView, PositionIterator};

pub trait CellType: Copy + Default + std::fmt::Debug + Eq + PartialEq + Send {}

pub trait CellularAutomaton: 'static {
    type Cell: CellType;

    fn name(&self) -> &str {
        "Cellular Automaton"
    }
}

pub trait CPUComputableAutomaton: CellularAutomaton {
    fn update_cell<'a>(grid: &GridView<'a, Self::Cell>) -> Self::Cell;

    fn update_grid(grid: &Grid<Self::Cell>) -> Grid<Self::Cell> {
        let dim = grid.dim();
        let mut new_data = Vec::with_capacity(dim.size() as usize);
        for pos in PositionIterator::new(*dim) {
            let new_cell = Self::update_cell(&grid.view(pos));
            new_data.push(new_cell);
        }
        Grid::from_data(new_data, *dim)
    }
}

pub trait GPUComputableAutomaton: CellularAutomaton + Send
where
    Self::Cell: Transcoder,
{
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;
    type PushConstants: Copy;

    fn vk_setup(device: &Arc<Device>) -> PipelineInfo<Self::Pipeline>;
    fn push_constants(grid: &Grid<Self::Cell>) -> Self::PushConstants;
}

pub trait Transcoder {
    fn encode(&self) -> u32;
    fn decode(id: u32) -> Self;
}

#[derive(Clone)]
pub struct PipelineInfo<P>
where
    P: ComputePipelineAbstract + Send + Sync + 'static,
{
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<P>,
}

pub trait TermDrawableAutomaton: CellularAutomaton {
    fn style(&self, state: &Self::Cell) -> StyledContent<char>;
}