use super::grid::GridView;
use crossterm::style::StyledContent;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipeline;

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

pub trait GPUCompute<S: Copy, Pl>: CellularAutomaton<S> {
    fn state_name(&self, state: &S) -> &str;

    fn update_gpu(&self, device: Arc<Device>) -> ComputePipeline<Pl>;
}
