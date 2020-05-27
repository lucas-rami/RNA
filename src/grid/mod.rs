// Standard library
use std::hash::Hash;

// CELL
pub mod grid;
pub mod grid_history;
pub mod grid_view;
pub use grid::Grid;
pub use grid_history::{GridDiff, GridHistory, GridHistoryOP};
pub use grid_view::GridView;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Position {
    x: u32,
    y: u32,
}
impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> u32 {
        self.y
    }
}

pub struct PositionIterator {
    dim: Dimensions,
    x: u32,
    y: u32,
}

impl PositionIterator {
    pub fn new(dim: Dimensions) -> Self {
        Self { dim, x: 0, y: 0 }
    }
}

impl Iterator for PositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.dim.height() {
            None
        } else {
            let ret = Position::new(self.x, self.y);
            if self.x == self.dim.width() - 1 {
                self.x = 0;
                self.y += 1;
            } else {
                self.x += 1;
            }
            Some(ret)
        }
    }
}

impl From<(u32, u32)> for Position {
    fn from(pos: (u32, u32)) -> Self {
        Position::new(pos.0, pos.1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Dimensions {
    width: u32,
    height: u32,
}
impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> u32 {
        self.width * self.height
    }

    #[inline]
    pub fn index(&self, pos: Position) -> usize {
        (pos.y() as usize) * (self.width as usize) + (pos.x() as usize)
    }
}

impl From<(u32, u32)> for Dimensions {
    fn from(dim: (u32, u32)) -> Self {
        Dimensions::new(dim.0, dim.1)
    }
}

#[derive(Copy, Clone)]
pub struct RelCoords {
    x: i32,
    y: i32,
}

impl RelCoords {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.y
    }
}

impl From<(i32, i32)> for RelCoords {
    fn from(coords: (i32, i32)) -> Self {
        RelCoords::new(coords.0, coords.1)
    }
}

pub const TOP: RelCoords = RelCoords { x: 0, y: -1 };
pub const TOP_RIGHT: RelCoords = RelCoords { x: 1, y: -1 };
pub const RIGHT: RelCoords = RelCoords { x: 1, y: 0 };
pub const BOTTOM_RIGHT: RelCoords = RelCoords { x: 1, y: 1 };
pub const BOTTOM: RelCoords = RelCoords { x: 0, y: 1 };
pub const BOTTOM_LEFT: RelCoords = RelCoords { x: -1, y: 1 };
pub const LEFT: RelCoords = RelCoords { x: -1, y: 0 };
pub const TOP_LEFT: RelCoords = RelCoords { x: -1, y: -1 };

pub const MOORE_NEIGHBORHOOD: [RelCoords; 8] = [
    TOP,
    TOP_RIGHT,
    RIGHT,
    BOTTOM_RIGHT,
    BOTTOM,
    BOTTOM_LEFT,
    LEFT,
    TOP_LEFT,
];

pub const NEUMANN_NEIGHBORHOOD: [RelCoords; 4] = [TOP, RIGHT, BOTTOM, LEFT];

#[cfg(test)]
mod tests {

    use super::*;
    use crate::simulator::advanced_channels;

    #[test]
    fn history_get_checkpoint() {
        let history = start_history(33, 16);
        assert_eq!(create_gen(0), history.get_gen(0).unwrap());
        assert_eq!(create_gen(16), history.get_gen(16).unwrap());
        assert_eq!(create_gen(32), history.get_gen(32).unwrap());
    }

    #[test]
    fn history_get_with_diffs() {
        let history = start_history(32, 16);
        assert_eq!(create_gen(1), history.get_gen(1).unwrap());
        assert_eq!(create_gen(15), history.get_gen(15).unwrap());
        assert_eq!(create_gen(20), history.get_gen(20).unwrap());
    }

    #[test]
    fn history_get_no_checkpoint() {
        let history = start_history(20, 0);
        assert_eq!(create_gen(0), history.get_gen(0).unwrap());
        assert_eq!(create_gen(8), history.get_gen(8).unwrap());
        assert_eq!(create_gen(20), history.get_gen(20).unwrap());
    }

    #[test]
    fn history_dispatch_block() {
        let history = start_history(0, 10);
        let (grid_master, grid_slave) = advanced_channels::twoway_channel();
        let grid_thrid_party = grid_master.create_third_party();
        std::thread::spawn(move || history.dispatch(grid_slave));
        std::thread::spawn(move || {
            for i in 1..5 {
                grid_thrid_party.send(GridHistoryOP::Push(create_gen(i)));
            }
        });

        let gens = vec![0, 1, 2, 3, 4];
        for gen in gens {
            let received_grid = grid_master.send_and_wait_for_response(GridHistoryOP::GetGen {
                gen,
                blocking: true,
            }).unwrap();
            assert_eq!(create_gen(gen), received_grid);
        }
    }

    #[test]
    fn history_dispatch() {
        let history = start_history(0, 0);
        let (grid_master, grid_slave) = advanced_channels::twoway_channel();

        std::thread::spawn(move || history.dispatch(grid_slave));

        // Initial generation should be available immediately
        let received_grid = grid_master.send_and_wait_for_response(GridHistoryOP::GetGen {
            gen: 0,
            blocking: false,
        }).unwrap();
        assert_eq!(create_gen(0), received_grid);

        // Generation 1 shouldn't be available
        let received_grid = grid_master.send_and_wait_for_response(GridHistoryOP::GetGen {
            gen: 1,
            blocking: false,
        });
        if let Some(_) = received_grid {
            panic!("Generation 1 shouldn't be available.")
        }

        // Pushing generation 1 and retrieving it should be possible
        grid_master.send(GridHistoryOP::Push(create_gen(1)));
        let received_grid = grid_master.send_and_wait_for_response(GridHistoryOP::GetGen {
            gen: 1,
            blocking: true,
        });
        if let Some(grid) = received_grid {
            assert_eq!(grid, create_gen(1));
        }
    }

    fn create_gen(gen: usize) -> Grid<u32> {
        let mut data = vec![
            0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32,
            0u32, 0u32,
        ];
        let idx = gen % data.len();
        data[idx] = (gen / data.len()) as u32 + 1;
        Grid::from_data(data, Dimensions::new(4, 4))
    }

    fn start_history(nb_gens: usize, f_check: usize) -> GridHistory<u32> {
        let mut history = GridHistory::new(&create_gen(0), f_check);
        for i in 1..(nb_gens + 1) {
            history.push(create_gen(i));
        }
        history
    }
}
