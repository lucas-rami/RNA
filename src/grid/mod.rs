// Standard library
use std::hash::Hash;

// CELL
pub mod grid;
pub mod grid_history;
pub mod grid_view;
pub use grid::Grid;
pub use grid_history::GridHistory;
pub use grid_view::GridView;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    x: u32,
    y: u32,
}
impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> u32 {
        self.y
    }
}

impl From<(u32, u32)> for Position {
    fn from(pos: (u32, u32)) -> Self {
        Position::new(pos.0, pos.1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Dimensions {
    width: u32,
    height: u32,
}
impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> u32 {
        self.width * self.height
    }

    #[inline]
    pub fn index(&self, pos: Position) -> usize {
        (pos.y() as usize) * (self.width as usize) + (pos.x() as usize)
    }
}

impl From<(u32, u32)> for Dimensions {
    fn from(dim: (u32, u32)) -> Self {
        Dimensions::new(dim.0, dim.1)
    }
}

#[derive(Copy, Clone)]
pub struct RelCoords {
    x: i32,
    y: i32,
}

impl RelCoords {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.y
    }
}

impl From<(i32, i32)> for RelCoords {
    fn from(coords: (i32, i32)) -> Self {
        RelCoords::new(coords.0, coords.1)
    }
}

pub const TOP: RelCoords = RelCoords { x: 0, y: -1 };
pub const TOP_RIGHT: RelCoords = RelCoords { x: 1, y: -1 };
pub const RIGHT: RelCoords = RelCoords { x: 1, y: 0 };
pub const BOTTOM_RIGHT: RelCoords = RelCoords { x: 1, y: 1 };
pub const BOTTOM: RelCoords = RelCoords { x: 0, y: 1 };
pub const BOTTOM_LEFT: RelCoords = RelCoords { x: -1, y: 1 };
pub const LEFT: RelCoords = RelCoords { x: -1, y: 0 };
pub const TOP_LEFT: RelCoords = RelCoords { x: -1, y: -1 };

pub const MOORE_NEIGHBORHOOD: [RelCoords; 8] = [
    TOP,
    TOP_RIGHT,
    RIGHT,
    BOTTOM_RIGHT,
    BOTTOM,
    BOTTOM_LEFT,
    LEFT,
    TOP_LEFT,
];

pub const NEUMANN_NEIGHBORHOOD: [RelCoords; 4] = [TOP, RIGHT, BOTTOM, LEFT];
