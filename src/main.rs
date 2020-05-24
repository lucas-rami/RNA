// External libraries
use vulkano::instance::{Instance, InstanceExtensions};

// CELL
mod automaton;
mod commands;
mod game_of_life;
mod grid;
mod heat_dispersion;
mod simulator;
mod terminal_ui;
use game_of_life::GameOfLife;
use simulator::Simulator;
use terminal_ui::TerminalUI;

fn main() -> () {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    let sim = Simulator::new_gpu_sim(
        "Conway GPU",
        GameOfLife::new(),
        &game_of_life::gosper_glider_gun(),
        instance,
    );
    let mut term_ui = TerminalUI::new(sim);
    term_ui.cmd_interpreter().unwrap();
}
