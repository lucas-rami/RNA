// External libraries
use vulkano::instance::{Instance, InstanceExtensions};

// CELL
mod commands;
mod game_of_life;
mod simulator;
mod terminal_ui;
use game_of_life::{GameOfLife, conway_canon};
use simulator::{Simulator, GPUSimulator};
// use terminal_ui::TerminalUI;

fn main() -> () {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    let mut gpu_sim = GPUSimulator::new(
        "Conway GPU",
        GameOfLife::new(),
        &conway_canon(),
        instance,
    );
    gpu_sim.run(1);
    // let mut term_ui = TerminalUI::new(game_of_life::conway_canon());
    // term_ui.cmd_interpreter().unwrap();
    // buffer_test();
}
