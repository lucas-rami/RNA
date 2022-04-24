// Standard library
use std::fmt::Debug;

// External libraries
use crossterm::style::StyledContent;

// Local
pub mod game_of_life;
use crate::universe::Universe;

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
