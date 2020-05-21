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

    pub fn cell(&self) -> &T {
        self.grid.get(self.pos)
    }

    pub fn get_relative(&self, coords: &RelCoords) -> &T {
        // @TODO to refactor once we have a formally defined notion of neighborhood
        let dim = self.grid.dim();
        if 1 <= self.pos.x()
            && self.pos.x() < dim.width()
            && 1 <= self.pos.y()
            && self.pos.y() < dim.height()
        {
            let x = {
                if coords.x() < 0 {
                    self.pos.x() - (coords.x().abs() as u32)
                } else {
                    self.pos.x() + (coords.x().abs() as u32)
                }
            };
            let y = {
                if coords.y() < 0 {
                    self.pos.y() - (coords.y().abs() as u32)
                } else {
                    self.pos.y() + (coords.y().abs() as u32)
                }
            };

            self.grid.get(Position::new(x, y))
        } else {
            &self.default
        }
    }

    pub fn get_relative_mul(&self, mul_coords: &[RelCoords]) -> Vec<&T> {
        mul_coords.iter().map(|x| self.get_relative(x)).collect()
    }
}
