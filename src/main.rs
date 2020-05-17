// External libraries
use vulkano::instance::{Instance, InstanceExtensions};

// CELL
mod commands;
mod game_of_life;
mod simulator;
// mod terminal_ui;
mod grid;
use game_of_life::{conway_canon, GameOfLife};
use simulator::Simulator;
// use terminal_ui::TerminalUI;

fn main() -> () {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    let sim = Simulator::new("Conway GPU", GameOfLife::new(), conway_canon());
    // let mut term_ui = TerminalUI::new(sim);
    // term_ui.cmd_interpreter().unwrap();
}
