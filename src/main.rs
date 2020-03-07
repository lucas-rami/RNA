use crossterm::{
    cursor, execute, queue,
    style::{style, Attribute, Color, PrintStyledContent, StyledContent},
    terminal, Result,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{stdout, Write};
use std::{thread, time};

trait Cells: Clone + Eq + Hash + PartialEq {
    fn default() -> Self;
    fn update_cell(grid: &Grid<Self>, row: usize, col: usize) -> Self;
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum ConwayGameOfLife {
    Dead = 0,
    Alive = 1,
}

impl Cells for ConwayGameOfLife {
    fn default() -> Self {
        Self::Dead
    }

    fn update_cell(grid: &Grid<Self>, row: usize, col: usize) -> Self {
        // All 8 neighbors
        let directions = [
            Neighbor::Top,
            Neighbor::TopRight,
            Neighbor::Right,
            Neighbor::BottomRight,
            Neighbor::Bottom,
            Neighbor::BottomLeft,
            Neighbor::Left,
            Neighbor::TopLeft,
        ];

        // Count the number of alive cells around us
        let mut nb_alive_neighbors = 0;
        for dir in directions.iter() {
            if let Self::Alive = grid.neighbor(row, col, dir) {
                nb_alive_neighbors += 1;
            }
        }

        // Apply the evolution rule
        match grid.get(row, col) {
            Self::Dead => {
                if nb_alive_neighbors == 3 {
                    Self::Alive
                } else {
                    Self::Dead
                }
            }
            Self::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
                    Self::Alive
                } else {
                    Self::Dead
                }
            }
        }
    }
}

enum Neighbor {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

struct Grid<C: Cells> {
    nb_rows: usize,
    nb_cols: usize,
    data: Vec<C>,
}

impl<C: Cells> Grid<C> {
    fn get(&self, row: usize, col: usize) -> &C {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Invalid grid index.")
        }
        &self.data[row * self.nb_cols + col]
    }

    fn new(nb_rows: usize, nb_cols: usize) -> Grid<C> {
        Grid {
            nb_rows,
            nb_cols,
            data: vec![C::default(); nb_rows * nb_cols],
        }
    }

    fn neighbor(&self, row: usize, col: usize, direction: &Neighbor) -> C {
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

struct CellularAutomaton<C: Cells> {
    grid: Grid<C>,
    display: HashMap<C, StyledContent<char>>,
}

impl<C: Cells> CellularAutomaton<C> {
    fn new(
        nb_rows: usize,
        nb_cols: usize,
        display: HashMap<C, StyledContent<char>>,
    ) -> CellularAutomaton<C> {
        CellularAutomaton {
            grid: Grid::new(nb_rows, nb_cols),
            display,
        }
    }

    fn print(&self) -> Result<()> {
        let mut stdout = stdout();
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        for row in 0..self.grid.nb_rows {
            queue!(stdout, cursor::MoveTo(0, row as u16))?;
            for col in 0..self.grid.nb_cols {
                let c = match self.display.get(self.grid.get(row, col)) {
                    Some(repr) => repr.clone(),
                    None => style('?'),
                };
                queue!(stdout, PrintStyledContent(c))?;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn run(&mut self) -> () {
        let mut new_data = vec![];
        for row in 0..self.grid.nb_rows {
            for col in 0..self.grid.nb_cols {
                new_data.push(C::update_cell(&self.grid, row, col));
            }
        }
        self.grid.data = new_data;
    }

    fn set_cell(&mut self, row: usize, col: usize, new_state: C) -> () {
        if self.grid.nb_rows <= row || self.grid.nb_cols <= col {
            panic!("Cell index is invalid.")
        }
        let idx = row * self.grid.nb_cols + col;
        self.grid.data[idx] = new_state
    }
}

fn main() -> Result<()> {
    let mut display = HashMap::new();
    display.insert(ConwayGameOfLife::Dead, style('·'));
    display.insert(
        ConwayGameOfLife::Alive,
        style('#').with(Color::Blue).attribute(Attribute::Bold),
    );

    let mut conway = CellularAutomaton::<ConwayGameOfLife>::new(10, 20, display);
    conway.set_cell(3, 4, ConwayGameOfLife::Alive);
    conway.set_cell(3, 5, ConwayGameOfLife::Alive);
    conway.set_cell(3, 6, ConwayGameOfLife::Alive);
    for _ in 0..100 {
        conway.print()?;
        conway.run();
        thread::sleep(time::Duration::from_millis(500));
    }

    Ok(())
}
