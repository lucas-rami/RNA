// CELL
use super::{Grid, Position, RelCoords};

pub struct GridView<'a, T: Copy + Default> {
    pos: Position,
    grid: &'a Grid<T>,
    default: T,
}

impl<'a, T: Copy + Default> GridView<'a, T> {
    pub fn new(grid: &'a Grid<T>, pos: Position) -> Self {
        Self {
            pos,
            grid,
            default: T::default(),
        }
    }

    pub fn state(&self) -> T {
        self.grid.get(self.pos)
    }

    pub fn get(&self, coords: &RelCoords) -> T {
        let row = {
            if coords.y() < 0 && (coords.y().abs() as u32) <= self.pos.y() {
                Some(self.pos.y() - (coords.y().abs() as u32))
            } else {
                let idx = (coords.y() as u32) + self.pos.y();
                if idx < self.grid.dim().height() {
                    Some(idx)
                } else {
                    None
                }
            }
        };

        let col = {
            if coords.x() < 0 && (coords.x.abs() as u32) <= self.pos.x() {
                Some(self.pos.x() - (coords.x.abs() as u32))
            } else {
                let idx = (coords.x() as u32) + self.pos.x();
                if idx < self.grid.dim().width() {
                    Some(idx)
                } else {
                    None
                }
            }
        };
        match row {
            Some(y) => match col {
                Some(x) => self.grid.get(Position::new(y, x)),
                None => self.default,
            },
            None => self.default,
        }
    }

    pub fn get_multiple(&self, mul_coords: &[RelCoords]) -> Vec<T> {
        mul_coords.iter().map(|x| self.get(x)).collect()
    }
}
