// External libraries
use crossterm::style::{style, Attribute, Color, StyledContent};

// Local
use crate::{
    automaton::{AutomatonCell, CPUCell, TermDrawableAutomaton},
    universe::{
        grid2d::{
            infinite_grid2d::InfiniteGrid2D,
            SCoordinates2D,
            {static_grid2d::StaticGrid2D, Coordinates2D, Size2D},
            {Neighbor2D, MOORE_NEIGHBORHOOD},
        },
        {CPUUniverse, Universe},
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

impl AutomatonCell for GameOfLife {
    type Neighbor = Neighbor2D;
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
            _ => panic!(format!("Decoding failed: unkwnon encoding {}.", id)),
        }
    }

    fn neighborhood() -> &'static [Self::Neighbor] {
        &MOORE_NEIGHBORHOOD
    }
}

impl CPUCell for GameOfLife {
    fn update<U: CPUUniverse<Cell = Self>>(&self, universe: &U, coords: U::Coordinates) -> Self {
        // Count the number of alive cells around us
        let mut nb_alive_neighbors = 0 as u32;
        for nbor in Self::neighborhood() {
            if let GameOfLife::Alive = universe.neighbor(coords.clone(), *nbor) {
                nb_alive_neighbors += 1;
            }
        }

        // Apply the evolution rule
        match self {
            GameOfLife::Dead => {
                if nb_alive_neighbors == 3 {
                    GameOfLife::Alive
                } else {
                    GameOfLife::Dead
                }
            }
            GameOfLife::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
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
    blinker.set(Coordinates2D(1, 2), GameOfLife::Alive);
    blinker.set(Coordinates2D(2, 2), GameOfLife::Alive);
    blinker.set(Coordinates2D(3, 2), GameOfLife::Alive);
    blinker
}

pub fn is_blinker(grid: &StaticGrid2D<GameOfLife>, flipped: bool) -> bool {
    let cell_is_valid = |pos: Coordinates2D, cell: GameOfLife| {
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

const PENTA_DECATHLON_ALIVE_SET: [Coordinates2D; 18] = [
    Coordinates2D(3, 6),
    Coordinates2D(3, 7),
    Coordinates2D(3, 8),
    Coordinates2D(3, 9),
    Coordinates2D(3, 10),
    Coordinates2D(3, 11),
    Coordinates2D(7, 6),
    Coordinates2D(7, 7),
    Coordinates2D(7, 8),
    Coordinates2D(7, 9),
    Coordinates2D(7, 10),
    Coordinates2D(7, 11),
    Coordinates2D(4, 5),
    Coordinates2D(5, 4),
    Coordinates2D(6, 5),
    Coordinates2D(4, 12),
    Coordinates2D(5, 13),
    Coordinates2D(6, 12),
];

pub fn penta_decathlon() -> StaticGrid2D<GameOfLife> {
    let mut penta_decathlon = StaticGrid2D::new_empty(Size2D(11, 18));
    for pos in &PENTA_DECATHLON_ALIVE_SET {
        penta_decathlon.set(*pos, GameOfLife::Alive);
    }
    penta_decathlon
}

pub fn is_penta_decathlon(grid: &StaticGrid2D<GameOfLife>) -> bool {
    let mut nb_alives = PENTA_DECATHLON_ALIVE_SET.len();
    for col_iter in grid.iter() {
        for (pos, cell) in col_iter {
            if cell == GameOfLife::Alive {
                if PENTA_DECATHLON_ALIVE_SET.contains(&pos) && nb_alives != 0 {
                    nb_alives -= 1;
                } else {
                    return false;
                }
            }
        }
    }
    nb_alives == 0
}

const LWSS_P0: [SCoordinates2D; 9] = [
    SCoordinates2D(0, 0),
    SCoordinates2D(3, 0),
    SCoordinates2D(4, -1),
    SCoordinates2D(0, -2),
    SCoordinates2D(4, -2),
    SCoordinates2D(1, -3),
    SCoordinates2D(2, -3),
    SCoordinates2D(3, -3),
    SCoordinates2D(4, -3),
];

const LWSS_P1: [SCoordinates2D; 12] = [
    SCoordinates2D(3, -1),
    SCoordinates2D(4, -1),
    SCoordinates2D(1, -2),
    SCoordinates2D(2, -2),
    SCoordinates2D(4, -2),
    SCoordinates2D(5, -2),
    SCoordinates2D(1, -3),
    SCoordinates2D(2, -3),
    SCoordinates2D(3, -3),
    SCoordinates2D(4, -3),
    SCoordinates2D(2, -4),
    SCoordinates2D(3, -4),
];

const LWSS_P2: [SCoordinates2D; 9] = [
    SCoordinates2D(2, -1),
    SCoordinates2D(3, -1),
    SCoordinates2D(4, -1),
    SCoordinates2D(5, -1),
    SCoordinates2D(1, -2),
    SCoordinates2D(5, -2),
    SCoordinates2D(5, -3),
    SCoordinates2D(1, -4),
    SCoordinates2D(4, -4),
];

const LWSS_P3: [SCoordinates2D; 12] = [
    SCoordinates2D(3, 0),
    SCoordinates2D(4, 0),
    SCoordinates2D(2, -1),
    SCoordinates2D(3, -1),
    SCoordinates2D(5, -1),
    SCoordinates2D(5, -1),
    SCoordinates2D(2, -2),
    SCoordinates2D(3, -2),
    SCoordinates2D(5, -2),
    SCoordinates2D(6, -2),
    SCoordinates2D(4, -3),
    SCoordinates2D(5, -3),
];

pub fn create_lwss(grid: &mut InfiniteGrid2D<GameOfLife>, base_coords: SCoordinates2D) {
    for c in &LWSS_P0 {
        let cell_coords = SCoordinates2D(base_coords.x() + c.x(), base_coords.y() + c.y());
        grid.set(cell_coords, GameOfLife::Alive);
    }
}

pub fn check_lwss(
    grid: &InfiniteGrid2D<GameOfLife>,
    base_coords: SCoordinates2D,
    gen: usize,
) -> bool {
    // Compute new base coordinates and select correct phase
    let nb_cycles = gen / 4;
    let phase_number = gen % 4;
    let coords = SCoordinates2D(
        base_coords.x() + 2 * (nb_cycles as isize),
        base_coords.y(),
    );

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
        let cell_coords = SCoordinates2D(coords.x() + c.x(), coords.y() + c.y());
        if grid.get(cell_coords) != GameOfLife::Alive {
            return false;
        }
    }
    true
}

// pub fn gosper_glider_gun() -> Grid<GameOfLife> {
//     let mut grid = Grid::new(Dimensions::new(100, 50));
//     grid = cascade!(
//         grid;
//         ..set(Position::new(1, 5), GameOfLife::Alive);
//         ..set(Position::new(1, 6), GameOfLife::Alive);
//         ..set(Position::new(2, 5), GameOfLife::Alive);
//         ..set(Position::new(2, 6), GameOfLife::Alive);
//         ..set(Position::new(11, 5), GameOfLife::Alive);
//         ..set(Position::new(11, 6), GameOfLife::Alive);
//         ..set(Position::new(11, 7), GameOfLife::Alive);
//         ..set(Position::new(12, 4), GameOfLife::Alive);
//         ..set(Position::new(12, 8), GameOfLife::Alive);
//         ..set(Position::new(13, 3), GameOfLife::Alive);
//         ..set(Position::new(13, 9), GameOfLife::Alive);
//         ..set(Position::new(14, 3), GameOfLife::Alive);
//         ..set(Position::new(14, 9), GameOfLife::Alive);
//         ..set(Position::new(15, 6), GameOfLife::Alive);
//         ..set(Position::new(16, 4), GameOfLife::Alive);
//         ..set(Position::new(16, 8), GameOfLife::Alive);
//         ..set(Position::new(17, 5), GameOfLife::Alive);
//         ..set(Position::new(17, 6), GameOfLife::Alive);
//         ..set(Position::new(17, 7), GameOfLife::Alive);
//         ..set(Position::new(18, 6), GameOfLife::Alive);
//         ..set(Position::new(21, 3), GameOfLife::Alive);
//         ..set(Position::new(21, 4), GameOfLife::Alive);
//         ..set(Position::new(21, 5), GameOfLife::Alive);
//         ..set(Position::new(22, 3), GameOfLife::Alive);
//         ..set(Position::new(22, 4), GameOfLife::Alive);
//         ..set(Position::new(22, 5), GameOfLife::Alive);
//         ..set(Position::new(23, 2), GameOfLife::Alive);
//         ..set(Position::new(23, 6), GameOfLife::Alive);
//         ..set(Position::new(25, 1), GameOfLife::Alive);
//         ..set(Position::new(25, 2), GameOfLife::Alive);
//         ..set(Position::new(25, 6), GameOfLife::Alive);
//         ..set(Position::new(25, 7), GameOfLife::Alive);
//         ..set(Position::new(35, 3), GameOfLife::Alive);
//         ..set(Position::new(35, 4), GameOfLife::Alive);
//         ..set(Position::new(36, 3), GameOfLife::Alive);
//         ..set(Position::new(36, 4), GameOfLife::Alive);
//     );
//     grid
// }

// pub fn r_pentomino() -> Grid<GameOfLife> {
//     let mut grid = Grid::new(Dimensions::new(201, 201));
//     grid = cascade!(
//         grid;
//         ..set(Position::new(100, 99), GameOfLife::Alive);
//         ..set(Position::new(101, 99), GameOfLife::Alive);
//         ..set(Position::new(99, 100), GameOfLife::Alive);
//         ..set(Position::new(100, 100), GameOfLife::Alive);
//         ..set(Position::new(100, 101), GameOfLife::Alive);
//     );
//     grid
// }
