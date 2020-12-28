// Standard library
use std::fmt::Debug;
use std::hash::Hash;

// CELL
// pub mod infinite_grid2d;
pub mod static_grid2d;

/// Size2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size2D(pub usize, pub usize);

impl Size2D {
    #[inline]
    pub fn total(&self) -> usize {
        self.0 * self.1
    }

    pub fn position(&self, idx: usize) -> Coordinates2D {
        if idx >= self.total() {
            panic!(format!(
                "Index should be less than {}, got {}.",
                self.total(),
                idx
            ));
        }
        Coordinates2D(idx % self.0, idx / self.0)
    }
}

impl From<(usize, usize)> for Size2D {
    fn from(tuple: (usize, usize)) -> Self {
        Size2D(tuple.0, tuple.1)
    }
}

/// Coordinates2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coordinates2D(pub usize, pub usize);

impl Coordinates2D {
    #[inline]
    pub fn x(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn y(&self) -> usize {
        self.1
    }

    pub fn idx(&self, size: &Size2D) -> usize {
        if !(self.0 < size.0 && self.1 < size.1) {
            panic!(format!(
                "Coordinates2D ({:?}) not within Size2D ({:?}).",
                *self, *size
            ))
        }
        self.0 + self.1 * size.0
    }
}

impl From<(usize, usize)> for Coordinates2D {
    fn from(tuple: (usize, usize)) -> Self {
        Coordinates2D(tuple.0, tuple.1)
    }
}

/// SCoordinates2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SCoordinates2D(pub isize, pub isize);

impl SCoordinates2D {
    #[inline]
    pub fn x(&self) -> isize {
        self.0
    }

    #[inline]
    pub fn y(&self) -> isize {
        self.1
    }

    #[inline]
    pub fn to_chunk_coordinates(&self, chunk_size_pow2: usize) -> Self {
        Self(self.0 >> chunk_size_pow2, self.1 >> chunk_size_pow2)
    }

    #[inline]
    pub fn to_coordinates_in_chunk(&self, chunk_size_pow2: usize) -> Coordinates2D {
        let mask = (1 << chunk_size_pow2) - 1 as isize;
        Coordinates2D((self.0 & mask) as usize, (self.1 & mask) as usize)
    }
}

/// Neighbor2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Neighbor2D(pub i32, pub i32);

impl Neighbor2D {
    pub fn max_one_axis_manhattan_distance(neighborhood: &[Neighbor2D]) -> usize {
        let mut max_manhattan_distance = 0;
        for n in neighborhood {
            let x = n.0.abs() as usize;
            let y = n.1.abs() as usize;
            let max = if x >= y { x } else { y };
            if max > max_manhattan_distance {
                max_manhattan_distance = max;
            }
        }
        max_manhattan_distance
    }
}

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
