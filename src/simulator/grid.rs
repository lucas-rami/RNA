#[derive(Clone)]
pub struct Grid<T: Clone> {
    dim: Dimensions,
    default: T,
    grid: Vec<T>,
}

impl<T: Clone> Grid<T> {
    pub fn new(dim: Dimensions, default: &T) -> Self {
        let grid = vec![default.clone(); dim.nb_rows * dim.nb_cols]; 
        Self {
            dim,
            default: default.clone(),
            grid,
        }
    }

    pub fn get(&self, pos: &Position) -> &T {
        if !self.pos_within_bounds(&pos) {
            panic!("Position not within grid.")
        }
        &self.grid[pos.row * self.dim.nb_cols + pos.col]
    }

    pub fn set(&mut self, pos: &Position, elem: T) -> () {
        if !self.pos_within_bounds(&pos) {
            panic!("Position not within grid.")
        }
        self.grid[pos.row * self.dim.nb_cols + pos.col] = elem;
    }

    pub fn view<'a>(&'a self, pos: Position) -> GridView<'a, T> {
        if !self.pos_within_bounds(&pos) {
            panic!("Position not within grid.")
        }
        GridView {
            pos,
            dim: &self.dim,
            default: &self.default,
            view: &self.grid,
        }
    }

    pub fn dim(&self) -> &Dimensions {
        &self.dim
    }

    fn pos_within_bounds(&self, pos: &Position) -> bool {
        pos.row < self.dim.nb_rows && pos.col < self.dim.nb_cols
    }
}

pub struct GridView<'a, T> {
    pos: Position,
    dim: &'a Dimensions,
    default: &'a T,
    view: &'a Vec<T>,
}

impl<'a, T> GridView<'a, T> {
    pub fn state(&self) -> &'a T {
        &self.view[self.pos.row * self.dim.nb_cols + self.pos.col]
    }
    
    pub fn get(&self, coords: RelCoords) -> &'a T {
        let row = {
            if coords.row < 0 && (coords.row.abs() as usize) <= self.pos.row {
                Some(self.pos.row - (coords.row.abs() as usize))
            } else {
                let idx = (coords.row as usize) + self.pos.row;
                if idx < self.dim.nb_rows {
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
                if idx < self.dim.nb_cols {
                    Some(idx)
                } else {
                    None
                }
            }
        };
        match row {
            Some(row_idx) => match col {
                Some(col_idx) => &self.view[row_idx * self.dim.nb_cols + col_idx],
                None => self.default,
            },
            None => self.default,
        }
    }

    pub fn get_multiple(&self, mul_coords: Vec<RelCoords>) -> Vec<&'a T> {
        mul_coords.into_iter().map(|x| self.get(x)).collect()
    }
}

#[derive(Clone)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(col: usize, row: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Clone)]
pub struct Dimensions {
    pub nb_rows: usize,
    pub nb_cols: usize,
}

impl Dimensions {
    pub fn new(nb_rows: usize, nb_cols: usize) -> Self {
        Self { nb_rows, nb_cols }
    }
}

#[derive(Clone)]
pub struct RelCoords {
    pub row: i32,
    pub col: i32,
}

impl RelCoords {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }
}
