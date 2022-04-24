use std::convert::TryInto;
// Standard library
use std::fmt::Debug;
use std::hash::Hash;

// Local
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
pub struct Loc2D(pub usize, pub usize);

impl Loc2D {
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
            panic!("Loc2D ({:?}) not within Size2D ({:?}).", *self, *size)
        }
        self.0 + self.1 * size.columns()
    }
}

impl From<ILoc2D> for Loc2D {
    fn from(loc: ILoc2D) -> Self {
        Loc2D(loc.0.try_into().unwrap(), loc.1.try_into().unwrap())
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
            let ret = LineIterator::new(Loc2D(0, self.line_idx), self.size.columns());
            self.line_idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}

/// LineIterator

pub struct LineIterator {
    coords: Loc2D,
    countdown: usize,
}

impl LineIterator {
    pub fn new(coords: Loc2D, countdown: usize) -> Self {
        Self { coords, countdown }
    }
}

impl Iterator for LineIterator {
    type Item = Loc2D;

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

/// ILoc2D

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ILoc2D(pub isize, pub isize);

impl ILoc2D {
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
    pub fn to_coordinates_in_chunk(&self, chunk_size_pow2: usize) -> Loc2D {
        let mask = (1 << chunk_size_pow2) - 1 as isize;
        Loc2D((self.0 & mask) as usize, (self.1 & mask) as usize)
    }
}

impl From<Loc2D> for ILoc2D {
    fn from(loc: Loc2D) -> Self {
        ILoc2D(loc.0.try_into().unwrap(), loc.1.try_into().unwrap())
    }
}
