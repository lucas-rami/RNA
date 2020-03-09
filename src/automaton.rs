use crossterm::{
    cursor, queue,
    style::{style, PrintStyledContent, StyledContent},
    Result,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{stdout, Write};

pub trait Cells: Clone + Eq + Hash + PartialEq {
    fn default() -> Self;
    fn update_cell(grid: &Grid<Self>, row: usize, col: usize) -> Self;
}

pub struct CellularAutomaton<C: Cells> {
    grid: Grid<C>,
    display: HashMap<C, StyledContent<char>>,
}

impl<C: Cells> CellularAutomaton<C> {
    
    pub fn new(
        nb_rows: usize,
        nb_cols: usize,
        display: HashMap<C, StyledContent<char>>,
    ) -> CellularAutomaton<C> {
        CellularAutomaton {
            grid: Grid::new(nb_rows, nb_cols),
            display,
        }
    }

    pub fn print_terminal(
        &self,
        term_offset: (u16, u16),
        auto_offset: (usize, usize),
        auto_size: (usize, usize),
    ) -> Result<()> {
        // Get handle to stdout
        let mut stdout = stdout();

        for row in auto_offset.1..auto_offset.1 + auto_size.1 {
            queue!(
                stdout,
                cursor::MoveTo(term_offset.0, term_offset.1 + (row as u16))
            )?;
            for col in auto_offset.0..auto_offset.0 + auto_size.0 {
                let c = match self.display.get(self.grid.get(row, col)) {
                    Some(repr) => repr.clone(),
                    None => style('?'),
                };
                queue!(stdout, PrintStyledContent(c))?;
            }
        }

        // Flush everything
        stdout.flush()?;
        Ok(())
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &C {
        self.grid.get(row, col)
    }

    pub fn run(&mut self) -> () {
        let mut new_data = vec![];
        for row in 0..self.grid.nb_rows {
            for col in 0..self.grid.nb_cols {
                new_data.push(C::update_cell(&self.grid, row, col));
            }
        }
        self.grid.data = new_data;
    }

    pub fn set_cell(&mut self, row: usize, col: usize, new_state: C) -> () {
        if self.grid.nb_rows <= row || self.grid.nb_cols <= col {
            panic!("Cell index is invalid.")
        }
        let idx = row * self.grid.nb_cols + col;
        self.grid.data[idx] = new_state
    }

    pub fn size(&self) -> (usize, usize) {
        (self.grid.nb_cols, self.grid.nb_rows)
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

pub struct Grid<C: Cells> {
    nb_rows: usize,
    nb_cols: usize,
    data: Vec<C>,
}

impl<C: Cells> Grid<C> {
    pub fn get(&self, row: usize, col: usize) -> &C {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Invalid grid index.")
        }
        &self.data[row * self.nb_cols + col]
    }

    pub fn new(nb_rows: usize, nb_cols: usize) -> Grid<C> {
        Grid {
            nb_rows,
            nb_cols,
            data: vec![C::default(); nb_rows * nb_cols],
        }
    }

    pub fn neighbor(&self, row: usize, col: usize, direction: &Neighbor) -> C {
        match direction {
            Neighbor::Top => {
                if row == 0 {
                    C::default()
                } else {
                    self.get(row - 1, col).clone()
                }
            }
            Neighbor::TopRight => {
                if row == 0 || col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get(row - 1, col + 1).clone()
                }
            }
            Neighbor::Right => {
                if col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get(row, col + 1).clone()
                }
            }
            Neighbor::BottomRight => {
                if row == self.nb_rows - 1 || col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get(row + 1, col + 1).clone()
                }
            }
            Neighbor::Bottom => {
                if row == self.nb_rows - 1 {
                    C::default()
                } else {
                    self.get(row + 1, col).clone()
                }
            }
            Neighbor::BottomLeft => {
                if row == self.nb_rows - 1 || col == 0 {
                    C::default()
                } else {
                    self.get(row + 1, col - 1).clone()
                }
            }
            Neighbor::Left => {
                if col == 0 {
                    C::default()
                } else {
                    self.get(row, col - 1).clone()
                }
            }
            Neighbor::TopLeft => {
                if row == 0 || col == 0 {
                    C::default()
                } else {
                    self.get(row - 1, col - 1).clone()
                }
            }
        }
    }
}
