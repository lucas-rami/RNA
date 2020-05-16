// CELL
use super::{Grid, Position, RelCoords};

pub struct GridView<'a, T: Clone + Default> {
    pos: Position,
    grid: &'a Grid<T>,
    default: T,
}

impl<'a, T: Clone + Default> GridView<'a, T> {
    pub fn new(grid: &'a Grid<T>, pos: Position) -> Self {
        Self { pos, grid, default: T::default() }
    }

    pub fn state(&self) -> &'a T {
        &self.grid.data[self.pos.row * self.grid.dim.nb_cols + self.pos.col]
    }

    pub fn get(&self, coords: RelCoords) -> &'a T {
        let row = {
            if coords.row < 0 && (coords.row.abs() as usize) <= self.pos.row {
                Some(self.pos.row - (coords.row.abs() as usize))
            } else {
                let idx = (coords.row as usize) + self.pos.row;
                if idx < self.grid.dim.nb_rows {
                    Some(idx)
                } else {
                    None
                }
            }
        };

        let col = {
            if coords.col < 0 && (coords.col.abs() as usize) <= self.pos.col {
                Some(self.pos.col - (coords.col.abs() as usize))
            } else {
                let idx = (coords.col as usize) + self.pos.col;
                if idx < self.grid.dim.nb_cols {
                    Some(idx)
                } else {
                    None
                }
            }
        };
        match row {
            Some(row_idx) => match col {
                Some(col_idx) => &self.grid.data[row_idx * self.grid.dim.nb_cols + col_idx],
                None => &self.default,
            },
            None => &self.default,
        }
    }

    pub fn get_multiple(&self, mul_coords: Vec<RelCoords>) -> Vec<&'a T> {
        mul_coords.into_iter().map(|x| self.get(x)).collect()
    }
}
