use super::grid::GridView;
use crossterm::style::StyledContent;

pub trait CellularAutomaton: Clone {
    fn all_states() -> Vec<Self>;
    
    fn update_cpu<'a>(&self, grid: &GridView<'a, Self>) -> Self;

    fn default() -> Self;

    fn name(&self) -> String {
        String::from("Cellular Automaton")
    }
}

pub trait TermDrawable: PartialEq + Eq + std::hash::Hash {
    fn style(&self) -> StyledContent<char>;
} 

pub trait GPUCompute: CellularAutomaton {
    fn update_gpu(&self) -> String;
}
