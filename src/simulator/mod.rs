// CELL
pub mod cpu_simulator;
pub mod gpu_simulator;
pub mod grid;

pub use cpu_simulator::CPUSimulator;
pub use gpu_simulator::{GPUSimulator, GPUComputableAutomaton};
use grid::{grid_view::GridView, Dimensions, Position};

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
    fn cell(&self, pos: &Position) -> &A::State;
    fn size(&self) -> &Dimensions;
    fn name(&self) -> &str;
    fn current_gen(&self) -> u64;
}
