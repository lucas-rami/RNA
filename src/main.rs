use crossterm::{cursor, execute, queue, style, terminal, Result};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{stdout, Write};

type CellID = u8;

struct CellData {
    repr: char,
}

#[derive(Eq, Hash, PartialEq)]
enum ConwayState {
    Alive,
    Dead,
}

struct Cells<T>
where
    T: Eq + Hash,
{
    states: HashMap<T, CellID>,
    data: Vec<CellData>,
}

impl<T> Cells<T>
where
    T: Eq + Hash,
{
    fn new(mappings: HashMap<T, CellData>) -> Cells<T> {
        let mut states: HashMap<T, CellID> = HashMap::new();
        let mut data = Vec::new();
        let mut id: CellID = 0;
        for (state, cell_data) in mappings {
            states.insert(state, id);
            data.push(cell_data);
            id += 1;
        }

        Cells { states, data }
    }

    fn id(&self, state: &T) -> CellID {
        match self.states.get(state) {
            Some(id) => *id,
            None => panic!("Invalid cell state."),
        }
    }

    fn data(&self, id: CellID) -> &CellData {
        &self.data[id as usize]
    }
}

struct Grid<'a, T>
where
    T: Eq + Hash,
{
    nb_rows: usize,
    nb_cols: usize,
    cells: &'a Cells<T>,
    default_cell: T,
    grid: Vec<CellID>,
}

impl<'a, T> Grid<'a, T>
where
    T: Eq + Hash,
{
    fn new(nb_rows: usize, nb_cols: usize, cells: &Cells<T>, default_cell: T) -> Grid<T> {
        let grid = vec![cells.id(&default_cell); nb_rows * nb_cols];
        Grid {
            nb_rows,
            nb_cols,
            cells,
            default_cell,
            grid,
        }
    }

    fn to_string(&self) -> Result<()> {
        let mut stdout = stdout();
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        let mut idx: usize = 0;
        for y in 0..self.nb_rows {
            queue!(stdout, cursor::MoveTo(0, y as u16))?;
            for _ in 0..self.nb_cols {
                let c = self.cells.data(self.grid[idx]).repr;
                queue!(stdout, style::Print(c.to_string()))?;
                idx += 1;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn set(&mut self, row: usize, col: usize, new_state: T) -> () {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Cell index is invalid.")
        }
        let idx = row * self.nb_cols + col;
        self.grid[idx] = self.cells.id(&new_state)
    }

    fn reset(&mut self) -> () {
        self.grid = vec![self.cells.id(&self.default_cell); self.nb_rows * self.nb_cols];
    }
}

fn main() -> Result<()> {
    // Create Conway cells
    let mut conway_cells = HashMap::new();
    conway_cells.insert(ConwayState::Alive, CellData { repr: '#' });
    conway_cells.insert(ConwayState::Dead, CellData { repr: '.' });
    let conway_cells = Cells::new(conway_cells);

    // Create grid and set some cells
    let mut conway_grid = Grid::new(10, 20, &conway_cells, ConwayState::Dead);
    conway_grid.set(0, 0, ConwayState::Alive);
    conway_grid.set(0, 1, ConwayState::Alive);
    conway_grid.set(1, 0, ConwayState::Alive);
    conway_grid.to_string()?;

    conway_grid.set(2, 0, ConwayState::Alive);
    conway_grid.to_string()?;

    Ok(())
}
