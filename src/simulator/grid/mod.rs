// CELL
pub use self::grid_view::GridView;

pub mod grid_view;

#[derive(Clone)]
pub struct Grid<T: Clone> {
    dim: Dimensions,
    default: T,
    data: Vec<T>,
}

impl<T: Clone> Grid<T> {
    pub fn new(dim: Dimensions, default: &T) -> Self {
        let data = vec![default.clone(); dim.nb_rows * dim.nb_cols];
        Self {
            dim,
            default: default.clone(),
            data,
        }
    }

    pub fn get(&self, pos: &Position) -> &T {
        if !self.pos_within_bounds(&pos) {
            panic!("Position not within grid.")
        }
        &self.data[pos.row * self.dim.nb_cols + pos.col]
    }

    pub fn set(&mut self, pos: &Position, elem: T) -> () {
        if !self.pos_within_bounds(&pos) {
            panic!("Position not within grid.")
        }
        self.data[pos.row * self.dim.nb_cols + pos.col] = elem;
    }

    pub fn view<'a>(&'a self, pos: Position) -> GridView<'a, T> {
        if !self.pos_within_bounds(&pos) {
            panic!("Position not within grid.")
        }
        GridView::new(self, pos)
    }

    pub fn dim(&self) -> &Dimensions {
        &self.dim
    }

    fn pos_within_bounds(&self, pos: &Position) -> bool {
        pos.row < self.dim.nb_rows && pos.col < self.dim.nb_cols
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
