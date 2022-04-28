// Standard library
use std::fmt::Debug;

// External libraries
use crossterm::style::StyledContent;

// Local
pub mod game_of_life;
pub mod von_neumann;
use crate::universe::{Universe, grid2d::ILoc2D};

pub trait Cell: Copy + Debug + Default + Eq + PartialEq + Send + Sync + 'static {
    type Location: Clone;
    type Encoded: Copy + Send + Sync;

    fn encode(&self) -> Self::Encoded;

    fn decode(encoded: &Self::Encoded) -> Self;

    fn neighborhood(loc: Self::Location) -> Vec<Self::Location>;

    fn update<U: Universe<Cell = Self, Location = Self::Location>>(
        &self,
        universe: &U,
        loc: U::Location,
    ) -> Self;
}

pub trait GPUCell: Cell {}

pub trait TermDrawableAutomaton: Cell {
    fn style(&self) -> StyledContent<char>;
}

#[inline]
fn moore_neighborhood(loc: ILoc2D) -> Vec<ILoc2D> {
    vec![
        ILoc2D(loc.0, loc.1 - 1),
        ILoc2D(loc.0 + 1, loc.1 - 1),
        ILoc2D(loc.0 + 1, loc.1),
        ILoc2D(loc.0 + 1, loc.1 + 1),
        ILoc2D(loc.0, loc.1 + 1),
        ILoc2D(loc.0 - 1, loc.1 + 1),
        ILoc2D(loc.0 - 1, loc.1),
        ILoc2D(loc.0 - 1, loc.1 - 1),
    ]
}

#[inline]
fn von_neumann_neighborhood(loc: ILoc2D) -> Vec<ILoc2D> {
    vec![
        ILoc2D(loc.0, loc.1 - 1),
        ILoc2D(loc.0 + 1, loc.1),
        ILoc2D(loc.0, loc.1 + 1),
        ILoc2D(loc.0 - 1, loc.1),
    ]
}
