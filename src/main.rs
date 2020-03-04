use std::collections::HashMap;
use std::hash::Hash;

type CellID = u8;

struct CellData {
    id: CellID,
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
    cells: HashMap<T, CellData>,
}

impl<T> Cells<T>
where
    T: Eq + Hash,
{
    fn new(cells: HashMap<T, CellData>) -> Cells<T> {
        // Check validity: uniqueness of ids
        Cells { cells: cells }
    }

    fn get(&self, state: &T) -> &CellData {
        self.cells
            .get(state)
            .expect("Cell state has no data attached.")
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
        let grid = vec![cells.get(&default_cell).id; nb_rows * nb_cols];
        Grid {
            nb_rows,
            nb_cols,
            cells,
            default_cell,
            grid,
        }
    }

    fn set(&mut self, row: usize, col: usize, new_state: T) -> () {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Cell index is invalid.")
        }
        let idx = row * self.nb_rows + col;
        self.grid[idx] = self.cells.get(&new_state).id
    }

    fn reset(&mut self) -> () {
        self.grid = vec![self.cells.get(&self.default_cell).id; self.nb_rows * self.nb_cols];
    }
}

fn main() {
    // Create Conway cells
    let mut conway_cells = HashMap::new();
    conway_cells.insert(ConwayState::Alive, CellData { id: 0 });
    conway_cells.insert(ConwayState::Dead, CellData { id: 1 });
    let conway_cells = Cells::new(conway_cells);

    // Create grid and set some cells
    let mut conway_grid = Grid::new(10, 20, &conway_cells, ConwayState::Dead);
    conway_grid.set(0, 0, ConwayState::Alive);
    conway_grid.set(0, 1, ConwayState::Alive);
    conway_grid.set(1, 0, ConwayState::Alive);

    // Reset the grid
    conway_grid.reset();
}
