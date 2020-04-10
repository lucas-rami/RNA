use crate::automaton::{Cells, CellularAutomaton, Neighbor, Operation};
use cascade::cascade;

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
    let mut conway = CellularAutomaton::new("Conway Canon", 100, 200); // (38, 11)
    conway = cascade!(
        conway;
        ..perform(Operation::SetCell(1, 5, GameOfLife::Alive));
        ..perform(Operation::SetCell(1, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(2, 5, GameOfLife::Alive));
        ..perform(Operation::SetCell(2, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(11, 5, GameOfLife::Alive));
        ..perform(Operation::SetCell(11, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(11, 7, GameOfLife::Alive));
        ..perform(Operation::SetCell(12, 4, GameOfLife::Alive));
        ..perform(Operation::SetCell(12, 8, GameOfLife::Alive));
        ..perform(Operation::SetCell(13, 3, GameOfLife::Alive));
        ..perform(Operation::SetCell(13, 9, GameOfLife::Alive));
        ..perform(Operation::SetCell(14, 3, GameOfLife::Alive));
        ..perform(Operation::SetCell(14, 9, GameOfLife::Alive));
        ..perform(Operation::SetCell(15, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(16, 4, GameOfLife::Alive));
        ..perform(Operation::SetCell(16, 8, GameOfLife::Alive));
        ..perform(Operation::SetCell(17, 5, GameOfLife::Alive));
        ..perform(Operation::SetCell(17, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(17, 7, GameOfLife::Alive));
        ..perform(Operation::SetCell(18, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(21, 3, GameOfLife::Alive));
        ..perform(Operation::SetCell(21, 4, GameOfLife::Alive));
        ..perform(Operation::SetCell(21, 5, GameOfLife::Alive));
        ..perform(Operation::SetCell(22, 3, GameOfLife::Alive));
        ..perform(Operation::SetCell(22, 4, GameOfLife::Alive));
        ..perform(Operation::SetCell(22, 5, GameOfLife::Alive));
        ..perform(Operation::SetCell(23, 2, GameOfLife::Alive));
        ..perform(Operation::SetCell(23, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(25, 1, GameOfLife::Alive));
        ..perform(Operation::SetCell(25, 2, GameOfLife::Alive));
        ..perform(Operation::SetCell(25, 6, GameOfLife::Alive));
        ..perform(Operation::SetCell(25, 7, GameOfLife::Alive));
        ..perform(Operation::SetCell(35, 3, GameOfLife::Alive));
        ..perform(Operation::SetCell(35, 4, GameOfLife::Alive));
        ..perform(Operation::SetCell(36, 3, GameOfLife::Alive));
        ..perform(Operation::SetCell(36, 4, GameOfLife::Alive));
        ..perform(Operation::LockInitialState);
    );
    conway
}
