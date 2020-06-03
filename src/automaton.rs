// Standard library
use std::marker::PhantomData;
use std::sync::Arc;

// External libraries
use crossterm::style::StyledContent;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipelineAbstract;

// CELL
use crate::grid::{Dimensions, Grid, GridView, Neighbor, PositionIterator};

pub trait Cell: Copy + Default + std::fmt::Debug + Eq + PartialEq + Send + 'static {}
pub trait TranscodableCell: Cell {
    fn encode(&self) -> u32;
    fn decode(id: u32) -> Self;
}

pub struct CellularAutomaton<C: Cell> {
    name: String,
    neighborhood: Neighborhood,
    _marker: PhantomData<C>,
}

impl<C: Cell> CellularAutomaton<C> {
    pub fn new(name: &str, neighbors: &'static [Neighbor]) -> Self {
        Self {
            name: String::from(name),
            neighborhood: Neighborhood::new(neighbors),
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name[..]
    }

    pub fn neighborhood(&self) -> &Neighborhood {
        &self.neighborhood
    }
}

pub struct Neighborhood {
    pub neighbors: &'static [Neighbor],
    pub max_manhattan_distance: u32,
}

impl Neighborhood {
    fn new(neighbors: &'static [Neighbor]) -> Self {
        let mut max_manhattan_distance = 0;
        for neighbor in neighbors.iter() {
            // Update maximum Manhattan distance
            let x = neighbor.x().abs() as u32;
            let y = neighbor.y().abs() as u32;
            if x < y && max_manhattan_distance < y {
                max_manhattan_distance = y;
            } else if max_manhattan_distance < x {
                max_manhattan_distance = x;
            }
        }
        Self {
            neighbors,
            max_manhattan_distance,
        }
    }

    pub fn get_offsets(&self, dim: &Dimensions) -> Vec<i32> {
        self.neighbors
            .iter()
            .map(|n| n.x() + n.y() * (dim.width() as i32))
            .collect()
    }
}

pub trait UpdateCPU: Cell {
    fn update_cell<'a>(grid: &GridView<'a, Self>) -> Self;

    fn update_grid(grid: &Grid<Self>) -> Grid<Self> {
        let dim = grid.dim();
        let mut new_data = Vec::with_capacity(dim.size() as usize);
        for pos in PositionIterator::new(*dim) {
            let new_cell = Self::update_cell(&grid.view(pos));
            new_data.push(new_cell);
        }
        Grid::from_data(new_data, *dim)
    }
}

pub trait UpdateGPU: TranscodableCell {
    type Pipeline: ComputePipelineAbstract + Send + Sync + 'static;
    type PushConstants: Copy;

    fn vk_setup(device: &Arc<Device>) -> PipelineInfo<Self::Pipeline>;
    fn push_constants(grid: &Grid<Self>) -> Self::PushConstants;
}

#[derive(Clone)]
pub struct PipelineInfo<P>
where
    P: ComputePipelineAbstract + Send + Sync + 'static,
{
    pub layout: Arc<UnsafeDescriptorSetLayout>,
    pub pipeline: Arc<P>,
}

pub trait TermDrawableAutomaton: Cell {
    fn style(&self) -> StyledContent<char>;
}
