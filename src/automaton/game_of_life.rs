// External libraries
use crossterm::style::{style, Attribute, Color, StyledContent};

// Local
use crate::{
    automaton::{Cell, TermDrawableAutomaton},
    universe::{
        grid2d::{
            infinite_grid2d::InfiniteGrid2D,
            ILoc2D,
            {static_grid2d::StaticGrid2D, Size2D},
        },
        Universe,
    },
};

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum GameOfLife {
    Dead,
    Alive,
}

impl Default for GameOfLife {
    fn default() -> Self {
        Self::Dead
    }
}

impl Cell for GameOfLife {
    type Location = ILoc2D;
    type Encoded = u32;

    fn encode(&self) -> Self::Encoded {
        match self {
            GameOfLife::Dead => 0,
            GameOfLife::Alive => 1,
        }
    }

    fn decode(id: &Self::Encoded) -> Self {
        match id {
            0 => GameOfLife::Dead,
            1 => GameOfLife::Alive,
            _ => panic!("Decoding failed: unkwnon encoding {}.", id),
        }
    }

    fn neighborhood(loc: ILoc2D) -> Vec<ILoc2D> {
        vec![
            ILoc2D(loc.0, loc.1 - 1),
            ILoc2D(loc.0 + 1, loc.1 - 1),
            ILoc2D(loc.0 + 1, loc.1),
            ILoc2D(loc.0 + 1, loc.1 + 1),
            ILoc2D(loc.0, loc.1 + 1),
            ILoc2D(loc.0 - 1, loc.1 + 1),
            ILoc2D(loc.0 - 1, loc.1),
            ILoc2D(loc.0 - 1, loc.1 - 1),
        ]
    }

    fn update<U: Universe<Cell = Self, Location = Self::Location>>(
        &self,
        universe: &U,
        loc: U::Location,
    ) -> Self {
        // Count the number of alive cells around us
        let mut n_alive_neighbors = 0 as u32;
        for nbor in Self::neighborhood(loc) {
            if let GameOfLife::Alive = universe.get(nbor) {
                n_alive_neighbors += 1;
            }
        }

        // Apply the evolution rule
        match self {
            GameOfLife::Dead => {
                if n_alive_neighbors == 3 {
                    GameOfLife::Alive
                } else {
                    GameOfLife::Dead
                }
            }
            GameOfLife::Alive => {
                if n_alive_neighbors == 2 || n_alive_neighbors == 3 {
                    GameOfLife::Alive
                } else {
                    GameOfLife::Dead
                }
            }
        }
    }
}

impl TermDrawableAutomaton for GameOfLife {
    fn style(&self) -> StyledContent<char> {
        match self {
            GameOfLife::Dead => style('·').with(Color::Grey),
            GameOfLife::Alive => style('#').with(Color::Green).attribute(Attribute::Bold),
        }
    }
}

pub fn blinker() -> StaticGrid2D<GameOfLife> {
    let mut blinker = StaticGrid2D::new_empty(Size2D(5, 5));
    blinker.set(ILoc2D(1, 2), GameOfLife::Alive);
    blinker.set(ILoc2D(2, 2), GameOfLife::Alive);
    blinker.set(ILoc2D(3, 2), GameOfLife::Alive);
    blinker
}

pub fn is_blinker(grid: &StaticGrid2D<GameOfLife>, flipped: bool) -> bool {
    let cell_is_valid = |pos: ILoc2D, cell: GameOfLife| {
        if flipped {
            if pos.1 >= 1 && pos.1 <= 3 && pos.0 == 2 {
                return cell == GameOfLife::Alive;
            }
        } else {
            if pos.0 >= 1 && pos.0 <= 3 && pos.1 == 2 {
                return cell == GameOfLife::Alive;
            }
        }
        cell == GameOfLife::Dead
    };

    // Check that every cell is valid
    for col_iter in grid.iter() {
        for (pos, cell) in col_iter {
            if !cell_is_valid(pos, cell) {
                return false;
            }
        }
    }
    true
}

const PENTA_DECATHLON_ALIVE_SET: [ILoc2D; 18] = [
    ILoc2D(3, 6),
    ILoc2D(3, 7),
    ILoc2D(3, 8),
    ILoc2D(3, 9),
    ILoc2D(3, 10),
    ILoc2D(3, 11),
    ILoc2D(7, 6),
    ILoc2D(7, 7),
    ILoc2D(7, 8),
    ILoc2D(7, 9),
    ILoc2D(7, 10),
    ILoc2D(7, 11),
    ILoc2D(4, 5),
    ILoc2D(5, 4),
    ILoc2D(6, 5),
    ILoc2D(4, 12),
    ILoc2D(5, 13),
    ILoc2D(6, 12),
];

pub fn penta_decathlon() -> StaticGrid2D<GameOfLife> {
    let mut penta_decathlon = StaticGrid2D::new_empty(Size2D(11, 18));
    for pos in &PENTA_DECATHLON_ALIVE_SET {
        penta_decathlon.set(*pos, GameOfLife::Alive);
    }
    penta_decathlon
}

pub fn is_penta_decathlon(grid: &StaticGrid2D<GameOfLife>) -> bool {
    let mut n_alives = PENTA_DECATHLON_ALIVE_SET.len();
    for col_iter in grid.iter() {
        for (pos, cell) in col_iter {
            if cell == GameOfLife::Alive {
                if PENTA_DECATHLON_ALIVE_SET.contains(&pos) && n_alives != 0 {
                    n_alives -= 1;
                } else {
                    return false;
                }
            }
        }
    }
    n_alives == 0
}

const LWSS_P0: [ILoc2D; 9] = [
    ILoc2D(0, 0),
    ILoc2D(3, 0),
    ILoc2D(4, -1),
    ILoc2D(0, -2),
    ILoc2D(4, -2),
    ILoc2D(1, -3),
    ILoc2D(2, -3),
    ILoc2D(3, -3),
    ILoc2D(4, -3),
];

const LWSS_P1: [ILoc2D; 12] = [
    ILoc2D(3, -1),
    ILoc2D(4, -1),
    ILoc2D(1, -2),
    ILoc2D(2, -2),
    ILoc2D(4, -2),
    ILoc2D(5, -2),
    ILoc2D(1, -3),
    ILoc2D(2, -3),
    ILoc2D(3, -3),
    ILoc2D(4, -3),
    ILoc2D(2, -4),
    ILoc2D(3, -4),
];

const LWSS_P2: [ILoc2D; 9] = [
    ILoc2D(2, -1),
    ILoc2D(3, -1),
    ILoc2D(4, -1),
    ILoc2D(5, -1),
    ILoc2D(1, -2),
    ILoc2D(5, -2),
    ILoc2D(5, -3),
    ILoc2D(1, -4),
    ILoc2D(4, -4),
];

const LWSS_P3: [ILoc2D; 12] = [
    ILoc2D(3, 0),
    ILoc2D(4, 0),
    ILoc2D(2, -1),
    ILoc2D(3, -1),
    ILoc2D(5, -1),
    ILoc2D(5, -1),
    ILoc2D(2, -2),
    ILoc2D(3, -2),
    ILoc2D(5, -2),
    ILoc2D(6, -2),
    ILoc2D(4, -3),
    ILoc2D(5, -3),
];

pub fn create_lwss(grid: &mut InfiniteGrid2D<GameOfLife>, base_coords: ILoc2D) {
    for c in &LWSS_P0 {
        let cell_coords = ILoc2D(base_coords.x() + c.x(), base_coords.y() + c.y());
        grid.set(cell_coords, GameOfLife::Alive);
    }
}

pub fn check_lwss(grid: &InfiniteGrid2D<GameOfLife>, base_coords: ILoc2D, gen: usize) -> bool {
    // Compute new base coordinates and select correct phase
    let n_cycles = gen / 4;
    let phase_number = gen % 4;
    let coords = ILoc2D(base_coords.x() + 2 * (n_cycles as isize), base_coords.y());

    // Check that the current phase is correct
    let phase = {
        if phase_number == 0 {
            LWSS_P0.iter()
        } else if phase_number == 1 {
            LWSS_P1.iter()
        } else if phase_number == 2 {
            LWSS_P2.iter()
        } else {
            LWSS_P3.iter()
        }
    };
    for c in phase {
        let cell_coords = ILoc2D(coords.x() + c.x(), coords.y() + c.y());
        if grid.get(cell_coords) != GameOfLife::Alive {
            return false;
        }
    }
    true
}
