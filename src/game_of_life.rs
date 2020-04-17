use crate::simulator::grid::{Grid, GridView, Position, RelCoords};
use crate::simulator::{
    automaton::{CellularAutomaton, TermDrawable},
    grid::Dimensions,
    Simulator,
};
use cascade::cascade;
use crossterm::style::{style, Attribute, Color, StyledContent};

#[derive(Clone, Eq, PartialEq, std::hash::Hash)]
pub enum GameOfLife {
    Dead,
    Alive,
}

impl CellularAutomaton for GameOfLife {
    fn all_states() -> Vec<Self> {
        vec![GameOfLife::Dead, GameOfLife::Alive]
    }
    fn update_cpu<'a>(&self, grid: &GridView<'a, Self>) -> Self {
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
            if let Self::Alive = cell {
                cnt + 1
            } else {
                cnt
            }
        });

        // Apply the evolution rule
        match self {
            Self::Dead => {
                if nb_alive_neighbors == 3 {
                    Self::Alive
                } else {
                    Self::Dead
                }
            }
            Self::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
                    Self::Alive
                } else {
                    Self::Dead
                }
            }
        }
    }
    fn default() -> Self {
        Self::Dead
    }

    fn name(&self) -> String {
        String::from("Conway's Game of Life")
    }
}

impl TermDrawable for GameOfLife {
    fn style(&self) -> StyledContent<char> {
        match self {
            Self::Dead => style('·').with(Color::Grey),
            Self::Alive => style('#').with(Color::Green).attribute(Attribute::Bold),
        }
    }
}

pub fn conway_canon() -> Simulator<GameOfLife> {
    let mut grid = Grid::new(Dimensions::new(100, 200));
    grid = cascade!(
        grid;
        ..set(&Position::new(1, 5), GameOfLife::Alive);
        ..set(&Position::new(1, 6), GameOfLife::Alive);
        ..set(&Position::new(2, 5), GameOfLife::Alive);
        ..set(&Position::new(2, 6), GameOfLife::Alive);
        ..set(&Position::new(11, 5), GameOfLife::Alive);
        ..set(&Position::new(11, 6), GameOfLife::Alive);
        ..set(&Position::new(11, 7), GameOfLife::Alive);
        ..set(&Position::new(12, 4), GameOfLife::Alive);
        ..set(&Position::new(12, 8), GameOfLife::Alive);
        ..set(&Position::new(13, 3), GameOfLife::Alive);
        ..set(&Position::new(13, 9), GameOfLife::Alive);
        ..set(&Position::new(14, 3), GameOfLife::Alive);
        ..set(&Position::new(14, 9), GameOfLife::Alive);
        ..set(&Position::new(15, 6), GameOfLife::Alive);
        ..set(&Position::new(16, 4), GameOfLife::Alive);
        ..set(&Position::new(16, 8), GameOfLife::Alive);
        ..set(&Position::new(17, 5), GameOfLife::Alive);
        ..set(&Position::new(17, 6), GameOfLife::Alive);
        ..set(&Position::new(17, 7), GameOfLife::Alive);
        ..set(&Position::new(18, 6), GameOfLife::Alive);
        ..set(&Position::new(21, 3), GameOfLife::Alive);
        ..set(&Position::new(21, 4), GameOfLife::Alive);
        ..set(&Position::new(21, 5), GameOfLife::Alive);
        ..set(&Position::new(22, 3), GameOfLife::Alive);
        ..set(&Position::new(22, 4), GameOfLife::Alive);
        ..set(&Position::new(22, 5), GameOfLife::Alive);
        ..set(&Position::new(23, 2), GameOfLife::Alive);
        ..set(&Position::new(23, 6), GameOfLife::Alive);
        ..set(&Position::new(25, 1), GameOfLife::Alive);
        ..set(&Position::new(25, 2), GameOfLife::Alive);
        ..set(&Position::new(25, 6), GameOfLife::Alive);
        ..set(&Position::new(25, 7), GameOfLife::Alive);
        ..set(&Position::new(35, 3), GameOfLife::Alive);
        ..set(&Position::new(35, 4), GameOfLife::Alive);
        ..set(&Position::new(36, 3), GameOfLife::Alive);
        ..set(&Position::new(36, 4), GameOfLife::Alive);
    );
    Simulator::new("Conway Cannon", &grid)
}
