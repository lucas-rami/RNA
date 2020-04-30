// External libraries
use vulkano::instance::{Instance, InstanceExtensions};

// CELL
mod commands;
mod game_of_life;
mod simulator;
mod terminal_ui;
use game_of_life::{GameOfLife, conway_canon, States};
use simulator::{Simulator, GPUSimulator};
use terminal_ui::TerminalUI;
use simulator::grid::{Position, Grid, Dimensions};


fn main() -> () {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    
    let mut simple_grid = Grid::new(Dimensions::new(3, 3), States::default());
    simple_grid.set(&Position::new(1, 1), States::Alive);
    
    let mut gpu_sim = GPUSimulator::new(
        "Conway GPU",
        GameOfLife::new(),
        &simple_grid,
        instance,
    );
    // gpu_sim.run(1);
    let mut term_ui = TerminalUI::new(gpu_sim);
    term_ui.cmd_interpreter().unwrap();
}
