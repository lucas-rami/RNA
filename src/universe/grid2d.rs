// Standard library
use std::fmt::Debug;
use std::hash::Hash;

// CELL
pub mod static_grid2d;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size2D(pub usize, pub usize);

impl Size2D {
    #[inline]
    pub fn total(&self) -> usize {
        self.0 * self.1
    }

    pub fn position(&self, idx: usize) -> Position2D {
        if idx >= self.total() {
            panic!(format!("Index should be less than {}, got {}.", self.total(), idx));
        }
        Position2D(idx % self.0, idx / self.0)
    }
}

impl From<(usize, usize)> for Size2D {
    fn from(tuple: (usize, usize)) -> Self {
        Size2D(tuple.0, tuple.1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position2D(pub usize, pub usize);

impl Position2D {
    pub fn idx(&self, size: &Size2D) -> usize {
        if !(self.0 < size.0 && self.1 < size.1) {
            panic!(format!(
                "Position2D ({:?}) not within Size2D ({:?}).",
                *self, *size
            ))
        }
        self.0 + self.1 * size.0
    }
}

impl From<(usize, usize)> for Position2D {
    fn from(tuple: (usize, usize)) -> Self {
        Position2D(tuple.0, tuple.1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Neighbor2D(pub i32, pub i32);

pub const MOORE_NEIGHBORHOOD: [Neighbor2D; 8] = [
    Neighbor2D(0, -1),
    Neighbor2D(1, -1),
    Neighbor2D(1, 0),
    Neighbor2D(1, 1),
    Neighbor2D(0, 1),
    Neighbor2D(-1, 1),
    Neighbor2D(-1, 0),
    Neighbor2D(-1, -1),
];

pub const VON_NEUMANN_NEIGHBORHOOD: [Neighbor2D; 4] = [
    Neighbor2D(0, -1),
    Neighbor2D(1, 0),
    Neighbor2D(0, 1),
    Neighbor2D(-1, 0),
];
