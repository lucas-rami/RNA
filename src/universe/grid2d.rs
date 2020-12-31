// Standard library
use std::fmt::Debug;
use std::hash::Hash;

// CELL
pub mod infinite_grid2d;
pub mod static_grid2d;

/// Size2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size2D(pub usize, pub usize);

impl Size2D {
    #[inline]
    pub fn columns(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn lines(&self) -> usize {
        self.1
    }

    #[inline]
    pub fn total(&self) -> usize {
        self.0 * self.1
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

    pub fn to_idx(&self, size: &Size2D) -> usize {
        if !(self.0 < size.columns() && self.1 < size.lines()) {
            panic!(format!(
                "Coordinates2D ({:?}) not within Size2D ({:?}).",
                *self, *size
            ))
        }
        self.0 + self.1 * size.columns()
    }
}

/// RectangleIterator

pub struct RectangleIterator {
    size: Size2D,
    line_idx: usize,
}

impl RectangleIterator {
    pub fn new(size: Size2D) -> Self {
        Self { size, line_idx: 0 }
    }
}

impl Iterator for RectangleIterator {
    type Item = LineIterator;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line_idx < self.size.lines() {
            let ret = LineIterator::new(Coordinates2D(0, self.line_idx), self.size.columns());
            self.line_idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

/// LineIterator

pub struct LineIterator {
    coords: Coordinates2D,
    countdown: usize,
}

impl LineIterator {
    pub fn new(coords: Coordinates2D, countdown: usize) -> Self {
        Self { coords, countdown }
    }
}

impl Iterator for LineIterator {
    type Item = Coordinates2D;

    fn next(&mut self) -> Option<Self::Item> {
        if self.countdown > 0 {
            self.coords.0 += 1;
            self.countdown -= 1;
            Some(self.coords)
        } else {
            None
        }
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
    pub fn to_universe_coordinates(&self, chunk_size_pow2: usize) -> Self {
        Self(self.0 << chunk_size_pow2, self.1 << chunk_size_pow2)
    }

    #[inline]
    pub fn to_coordinates_in_chunk(&self, chunk_size_pow2: usize) -> Coordinates2D {
        let mask = (1 << chunk_size_pow2) - 1 as isize;
        Coordinates2D((self.0 & mask) as usize, (self.1 & mask) as usize)
    }
}

/// Neighbor2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Neighbor2D(pub isize, pub isize);

impl Neighbor2D {
    #[inline]
    pub fn x(&self) -> isize {
        self.0
    }

    #[inline]
    pub fn y(&self) -> isize {
        self.1
    }

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
