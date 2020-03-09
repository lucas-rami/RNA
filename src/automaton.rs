pub trait Cells: Clone + Eq + std::hash::Hash + PartialEq {
    fn default() -> Self;
    fn update_cell(grid: &CellularAutomaton<Self>, row: usize, col: usize) -> Self;
}

pub struct CellularAutomaton<C: Cells> {
    nb_rows: usize,
    nb_cols: usize,
    data: Vec<C>,
}

impl<C: Cells> CellularAutomaton<C> {
    pub fn new(nb_rows: usize, nb_cols: usize) -> CellularAutomaton<C> {
        CellularAutomaton {
            nb_rows,
            nb_cols,
            data: vec![C::default(); nb_rows * nb_cols],
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &C {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Invalid grid index.")
        }
        &self.data[row * self.nb_cols + col]
    }

    pub fn neighbor(&self, row: usize, col: usize, direction: &Neighbor) -> C {
        match direction {
            Neighbor::Top => {
                if row == 0 {
                    C::default()
                } else {
                    self.get_cell(row - 1, col).clone()
                }
            }
            Neighbor::TopRight => {
                if row == 0 || col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get_cell(row - 1, col + 1).clone()
                }
            }
            Neighbor::Right => {
                if col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get_cell(row, col + 1).clone()
                }
            }
            Neighbor::BottomRight => {
                if row == self.nb_rows - 1 || col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get_cell(row + 1, col + 1).clone()
                }
            }
            Neighbor::Bottom => {
                if row == self.nb_rows - 1 {
                    C::default()
                } else {
                    self.get_cell(row + 1, col).clone()
                }
            }
            Neighbor::BottomLeft => {
                if row == self.nb_rows - 1 || col == 0 {
                    C::default()
                } else {
                    self.get_cell(row + 1, col - 1).clone()
                }
            }
            Neighbor::Left => {
                if col == 0 {
                    C::default()
                } else {
                    self.get_cell(row, col - 1).clone()
                }
            }
            Neighbor::TopLeft => {
                if row == 0 || col == 0 {
                    C::default()
                } else {
                    self.get_cell(row - 1, col - 1).clone()
                }
            }
        }
    }

    pub fn run(&mut self) -> () {
        let mut new_data = vec![];
        for row in 0..self.nb_rows {
            for col in 0..self.nb_cols {
                new_data.push(C::update_cell(&self, row, col));
            }
        }
        self.data = new_data;
    }

    pub fn set_cell(&mut self, row: usize, col: usize, new_state: C) -> () {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Cell index is invalid.")
        }
        let idx = row * self.nb_cols + col;
        self.data[idx] = new_state
    }

    pub fn set_cells(&mut self, cells: &mut Vec<(usize, usize, C)>) -> () {
        loop {
            match cells.pop() {
                Some(cell) => self.set_cell(cell.0, cell.1, cell.2),
                None => break
            }
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.nb_cols, self.nb_rows)
    }
}

pub enum Neighbor {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}
