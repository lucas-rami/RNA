// Standard library
use std::fmt::Debug;

// External libraries
use crossterm::style::StyledContent;

// CELL
pub mod game_of_life;
use crate::universe::CPUUniverse;

pub trait AutomatonCell: Copy + Debug + Default + Eq + PartialEq + Send + Sync + 'static {
    type Neighbor;
    type Encoded: Copy + Send + Sync;

    fn encode(&self) -> Self::Encoded;
    fn decode(encoded: &Self::Encoded) -> Self;

    fn neighborhood() -> &'static [Self::Neighbor];
}

pub trait CPUCell: AutomatonCell {
    fn update<U: CPUUniverse<Cell = Self, Neighbor = Self::Neighbor>>(
        &self,
        universe: &U,
        pos: &U::Position,
    ) -> Self;
}

pub trait GPUCell: AutomatonCell {}

pub trait TermDrawableAutomaton: AutomatonCell {
    fn style(&self) -> StyledContent<char>;
}
