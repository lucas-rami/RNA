// CELL
pub mod cpu;
pub mod gpu;
use crate::grid::{Dimensions, GridView, Position};

pub trait CellularAutomaton {
    type State: Copy + Default;

    fn update_cpu<'a>(&self, grid: &GridView<'a, Self::State>) -> Self::State;

    fn name(&self) -> &str {
        "Cellular Automaton"
    }
}

pub trait Simulator<A: CellularAutomaton> {
    fn run(&mut self, nb_gens: u64) -> ();
    fn automaton(&self) -> &A;
    fn cell(&self, pos: Position) -> A::State;
    fn size(&self) -> &Dimensions;
    fn name(&self) -> &str;
    fn current_gen(&self) -> u64;
}
