// Standard library
use std::collections::HashMap;

// External libraries
use cascade::cascade;
use crossterm::style::{style, Attribute, Color, StyledContent};

// CELL
use crate::simulator::grid::{Grid, GridView, Position, RelCoords};
use crate::simulator::{grid::Dimensions, CPUSimulator, CellularAutomaton};
use crate::terminal_ui::TerminalAutomaton;

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash)]
pub enum GOLStates {
    Dead,
    Alive,
}

pub struct GameOfLife {
    name: &'static str,
    style_map: HashMap<GOLStates, StyledContent<char>>,
}

impl GameOfLife {
    pub fn new() -> Self {
        let mut style_map = HashMap::new();
        style_map.insert(GOLStates::Dead, style('·').with(Color::Grey));
        style_map.insert(
            GOLStates::Alive,
            style('#').with(Color::Green).attribute(Attribute::Bold),
        );
        Self {
            name: "Conway's Game of Life",
            style_map,
        }
    }
}

impl CellularAutomaton<GOLStates> for GameOfLife {
    fn all_states(&self) -> Vec<GOLStates> {
        vec![GOLStates::Dead, GOLStates::Alive]
    }

    fn update_cpu<'a>(&self, grid: &GridView<'a, GOLStates>) -> GOLStates {
        // Count the number of alive cells around us
        let neighbors = vec![
            RelCoords::new(-1, -1),
            RelCoords::new(-1, 0),
            RelCoords::new(-1, 1),
            RelCoords::new(0, 1),
            RelCoords::new(1, 1),
            RelCoords::new(1, 0),
            RelCoords::new(1, -1),
            RelCoords::new(0, -1),
        ];
        let nb_alive_neighbors = grid.get_multiple(neighbors).iter().fold(0, |cnt, cell| {
            if let GOLStates::Alive = cell {
                cnt + 1
            } else {
                cnt
            }
        });

        // Apply the evolution rule
        match grid.state() {
            GOLStates::Dead => {
                if nb_alive_neighbors == 3 {
                    GOLStates::Alive
                } else {
                    GOLStates::Dead
                }
            }
            GOLStates::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
                    GOLStates::Alive
                } else {
                    GOLStates::Dead
                }
            }
        }
    }
    fn default(&self) -> GOLStates {
        GOLStates::Dead
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl TerminalAutomaton<GOLStates> for GameOfLife {
    fn style(&self, state: &GOLStates) -> &StyledContent<char> {
        &self.style_map.get(state).unwrap()
    }
}

pub fn conway_canon() -> CPUSimulator<GOLStates, GameOfLife> {
    let gol = GameOfLife::new();
    let mut grid = Grid::new(Dimensions::new(100, 200), &gol.default());
    grid = cascade!(
        grid;
        ..set(&Position::new(1, 5), GOLStates::Alive);
        ..set(&Position::new(1, 6), GOLStates::Alive);
        ..set(&Position::new(2, 5), GOLStates::Alive);
        ..set(&Position::new(2, 6), GOLStates::Alive);
        ..set(&Position::new(11, 5), GOLStates::Alive);
        ..set(&Position::new(11, 6), GOLStates::Alive);
        ..set(&Position::new(11, 7), GOLStates::Alive);
        ..set(&Position::new(12, 4), GOLStates::Alive);
        ..set(&Position::new(12, 8), GOLStates::Alive);
        ..set(&Position::new(13, 3), GOLStates::Alive);
        ..set(&Position::new(13, 9), GOLStates::Alive);
        ..set(&Position::new(14, 3), GOLStates::Alive);
        ..set(&Position::new(14, 9), GOLStates::Alive);
        ..set(&Position::new(15, 6), GOLStates::Alive);
        ..set(&Position::new(16, 4), GOLStates::Alive);
        ..set(&Position::new(16, 8), GOLStates::Alive);
        ..set(&Position::new(17, 5), GOLStates::Alive);
        ..set(&Position::new(17, 6), GOLStates::Alive);
        ..set(&Position::new(17, 7), GOLStates::Alive);
        ..set(&Position::new(18, 6), GOLStates::Alive);
        ..set(&Position::new(21, 3), GOLStates::Alive);
        ..set(&Position::new(21, 4), GOLStates::Alive);
        ..set(&Position::new(21, 5), GOLStates::Alive);
        ..set(&Position::new(22, 3), GOLStates::Alive);
        ..set(&Position::new(22, 4), GOLStates::Alive);
        ..set(&Position::new(22, 5), GOLStates::Alive);
        ..set(&Position::new(23, 2), GOLStates::Alive);
        ..set(&Position::new(23, 6), GOLStates::Alive);
        ..set(&Position::new(25, 1), GOLStates::Alive);
        ..set(&Position::new(25, 2), GOLStates::Alive);
        ..set(&Position::new(25, 6), GOLStates::Alive);
        ..set(&Position::new(25, 7), GOLStates::Alive);
        ..set(&Position::new(35, 3), GOLStates::Alive);
        ..set(&Position::new(35, 4), GOLStates::Alive);
        ..set(&Position::new(36, 3), GOLStates::Alive);
        ..set(&Position::new(36, 4), GOLStates::Alive);
    );
    CPUSimulator::new("Conway Cannon", gol, &grid)
}
