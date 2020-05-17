use super::{Dimensions, GridView, Position};

#[derive(Clone)]
pub struct Grid<T: Copy + Default> {
    dim: Dimensions,
    data: Vec<T>,
}

impl<T: Copy + Default> Grid<T> {
    pub fn new(dim: Dimensions) -> Self {
        let data = vec![T::default(); dim.size() as usize];
        Self { dim, data }
    }

    pub fn from_data(dim: Dimensions, data: Vec<T>) -> Self {
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

    pub fn switch_data(&mut self, new_data: Vec<T>) -> () {
        self.data = new_data
    }

    fn pos_within_bounds(&self, pos: Position) -> bool {
        pos.y() < self.dim.height() && pos.x() < self.dim.width()
    }
}

const ERR_POSITION: &str = "Position not within grid.";