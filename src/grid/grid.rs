// Standard library
use std::sync::Arc;

// External library
use vulkano::buffer::CpuAccessibleBuffer;

// CELL
use super::{Dimensions, GridDiff, GridView, Position};
use crate::simulator::Transcoder;

#[derive(Clone)]
pub struct Grid<T: Copy + Default + Eq + PartialEq> {
    dim: Dimensions,
    data: Vec<T>,
}

impl<T: Copy + Default + Eq + PartialEq> Grid<T> {
    pub fn new(dim: Dimensions) -> Self {
        let data = vec![T::default(); dim.size() as usize];
        Self { dim, data }
    }

    pub fn from_data(data: Vec<T>, dim: Dimensions) -> Self {
        if data.len() != dim.size() as usize {
            panic!("Vector length does not correspond to dimensions.")
        }
        Self { dim, data }
    }

    pub fn get(&self, pos: Position) -> T {
        if !self.pos_within_bounds(pos) {
            panic!(ERR_POSITION)
        }
        self.data[self.dim.index(pos)]
    }

    pub fn set(&mut self, pos: Position, elem: T) -> () {
        if !self.pos_within_bounds(pos) {
            panic!(ERR_POSITION)
        }
        self.data[self.dim.index(pos)] = elem;
    }

    pub fn view<'a>(&'a self, pos: Position) -> GridView<'a, T> {
        if !self.pos_within_bounds(pos) {
            panic!(ERR_POSITION)
        }
        GridView::new(self, pos)
    }

    pub fn dim(&self) -> &Dimensions {
        &self.dim
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
        self.data.iter()
    }

    pub fn apply_diffs(&mut self, diffs: GridDiff<T>) {
        for (pos, new_cell) in diffs.iter() {
            self.set(*pos, *new_cell);
        }
    }

    fn pos_within_bounds(&self, pos: Position) -> bool {
        pos.y() < self.dim.height() && pos.x() < self.dim.width()
    }
}

impl<T: Copy + Default + Eq + PartialEq + Transcoder> Grid<T> {
    pub fn encode(&self) -> Vec<u32> {
        let mut encoded = Vec::with_capacity(self.dim.size() as usize);
        for state in self.iter() {
            encoded.push(state.encode());
        }
        encoded
    }

    pub fn decode(encoded: Arc<CpuAccessibleBuffer<[u32]>>, dim: &Dimensions) -> Grid<T> {
        let size = dim.size() as usize;
        let raw_data = encoded.read().unwrap();
        let mut decoded = Vec::with_capacity(size);
        for idx in 0..size {
            decoded.push(T::decode(raw_data[idx]));
        }
        Grid::from_data(decoded, *dim)
    }
}

const ERR_POSITION: &str = "Position not within grid.";
