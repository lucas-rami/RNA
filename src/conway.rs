use crate::automaton::{Cells, CellularAutomaton, Neighbor};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum GameOfLife {
    Dead = 0,
    Alive = 1,
}

impl Cells for GameOfLife {
    fn default() -> Self {
        Self::Dead
    }

    fn update_cell(automaton: &CellularAutomaton<Self>, row: usize, col: usize) -> Self {
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
            if let Self::Alive = automaton.neighbor(row, col, dir) {
                nb_alive_neighbors += 1;
            }
        }

        // Apply the evolution rule
        match automaton.get_cell(row, col) {
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

pub fn conway_canon() -> CellularAutomaton<GameOfLife> {
    let mut conway = CellularAutomaton::new(200, 100); // (38, 11)
    conway.set_cells(&mut vec![
        // Left square
        (5, 1, GameOfLife::Alive),
        (6, 1, GameOfLife::Alive),
        (5, 2, GameOfLife::Alive),
        (6, 2, GameOfLife::Alive),
        // Canon left
        (5, 11, GameOfLife::Alive),
        (6, 11, GameOfLife::Alive),
        (7, 11, GameOfLife::Alive),
        (4, 12, GameOfLife::Alive),
        (8, 12, GameOfLife::Alive),
        (3, 13, GameOfLife::Alive),
        (9, 13, GameOfLife::Alive),
        (3, 14, GameOfLife::Alive),
        (9, 14, GameOfLife::Alive),
        (6, 15, GameOfLife::Alive),
        (4, 16, GameOfLife::Alive),
        (8, 16, GameOfLife::Alive),
        (5, 17, GameOfLife::Alive),
        (6, 17, GameOfLife::Alive),
        (7, 17, GameOfLife::Alive),
        (6, 18, GameOfLife::Alive),
        // Canon right
        (3, 21, GameOfLife::Alive),
        (4, 21, GameOfLife::Alive),
        (5, 21, GameOfLife::Alive),
        (3, 22, GameOfLife::Alive),
        (4, 22, GameOfLife::Alive),
        (5, 22, GameOfLife::Alive),
        (2, 23, GameOfLife::Alive),
        (6, 23, GameOfLife::Alive),
        (1, 25, GameOfLife::Alive),
        (2, 25, GameOfLife::Alive),
        (6, 25, GameOfLife::Alive),
        (7, 25, GameOfLife::Alive),
        // Right square
        (3, 35, GameOfLife::Alive),
        (4, 35, GameOfLife::Alive),
        (3, 36, GameOfLife::Alive),
        (4, 36, GameOfLife::Alive),
    ]);
    conway
}
