use super::grid::GridView;

pub trait CellularAutomaton: Clone {
    fn update_cpu<'a>(&self, grid: &GridView<'a, Self>) -> Self;

    fn default() -> Self;

    fn name(&self) -> String {
        String::from("Cellular Automaton")
    }
}

pub trait GPUCompute: CellularAutomaton {
    fn update_gpu(&self) -> String;
}
