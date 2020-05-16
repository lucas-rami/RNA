// External libraries
use vulkano::instance::{Instance, InstanceExtensions};

// CELL
mod commands;
mod game_of_life;
mod simulator;
mod terminal_ui;
use game_of_life::{conway_canon, GameOfLife};
use simulator::gpu::GPUSimulator;
use terminal_ui::TerminalUI;

fn main() -> () {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    let gpu_sim = GPUSimulator::new("Conway GPU", GameOfLife::new(), &conway_canon(), instance);
    let mut term_ui = TerminalUI::new(gpu_sim);
    term_ui.cmd_interpreter().unwrap();
}
