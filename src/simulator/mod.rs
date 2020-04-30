// CELL
pub mod cpu_simulator;
pub mod gpu_simulator;
pub mod grid;

pub use cpu_simulator::CPUSimulator;
pub use gpu_simulator::{GPUSimulator, GPUCompute};
use grid::{grid_view::GridView, Dimensions, Position};

pub trait CellularAutomaton<S: Copy + Default> {
    fn all_states(&self) -> Vec<S>;

    fn update_cpu<'a>(&self, grid: &GridView<'a, S>) -> S;

    fn name(&self) -> &str {
        "Cellular Automaton"
    }
}

pub trait Simulator<S: Copy + Default, C: CellularAutomaton<S>> {
    fn run(&mut self, nb_gens: u64) -> ();
    fn automaton(&self) -> &C;
    fn cell(&self, pos: &Position) -> &S;
    fn size(&self) -> &Dimensions;
    fn name(&self) -> &str;
    fn current_gen(&self) -> u64;
}
