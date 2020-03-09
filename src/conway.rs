use crate::automaton::{Cells, Neighbor, Grid};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum GameOfLife {
    Dead = 0,
    Alive = 1,
}

impl Cells for GameOfLife {
    fn default() -> Self {
        Self::Dead
    }

    fn update_cell(grid: &Grid<Self>, row: usize, col: usize) -> Self {
        // All 8 neighbors
        let directions = [
            Neighbor::Top,
            Neighbor::TopRight,
            Neighbor::Right,
            Neighbor::BottomRight,
            Neighbor::Bottom,
            Neighbor::BottomLeft,
            Neighbor::Left,
            Neighbor::TopLeft,
        ];

        // Count the number of alive cells around us
        let mut nb_alive_neighbors = 0;
        for dir in directions.iter() {
            if let Self::Alive = grid.neighbor(row, col, dir) {
                nb_alive_neighbors += 1;
            }
        }

        // Apply the evolution rule
        match grid.get(row, col) {
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
}