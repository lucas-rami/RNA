use super::grid::GridView;
use crossterm::style::StyledContent;

pub trait CellularAutomaton<S: Copy> {
    fn all_states(&self) -> Vec<S>;

    fn update_cpu<'a>(&self, grid: &GridView<'a, S>) -> S;

    fn default(&self) -> S;

    fn name(&self) -> &str {
        "Cellular Automaton"
    }
}

pub trait TermDrawableAutomaton<S: Copy>: CellularAutomaton<S> {
    fn style(&self, state: &S) -> &StyledContent<char>;
}

pub trait GPUCompute<S: Copy>: CellularAutomaton<S> {
    fn update_gpu(&self) -> String;
}
