pub mod automaton;
pub mod grid;

use automaton::CellularAutomaton;
use grid::{Dimensions, Grid, Position};

pub struct Simulator<C: CellularAutomaton> {
    name: String,
    init_state: Grid<C>,
    grid: Grid<C>,
    current_gen: u64,
}

impl<C: CellularAutomaton> Simulator<C> {
    pub fn new(name: &str, grid: &Grid<C>) -> Self {
        Self {
            name: String::from(name),
            init_state: grid.clone(),
            grid: grid.clone(),
            current_gen: 0,
        }
    }

    pub fn run(&mut self, nb_gens: u64) -> () {
        for _ in 0..nb_gens {
            let dim = self.grid.dim();
            let mut new_grid = Grid::new(dim.clone());
            for row in 0..dim.nb_rows {
                for col in 0..dim.nb_cols {
                    let pos = Position::new(col, row);
                    let view = self.grid.view(pos.clone());
                    let new_state = self.grid.get(&pos).update_cpu(&view);
                    new_grid.set(&pos, new_state);
                }
            }
            self.grid = new_grid;
        }
        self.current_gen += nb_gens
    }

    pub fn get_cell(&self, pos: &Position) -> &C {
        self.grid.get(pos)
    }

    pub fn size(&self) -> &Dimensions {
        self.init_state.dim()
    }

    pub fn get_name(&self) -> &str {
        &self.name[..]
    }

    pub fn current_gen(&self) -> u64 {
        self.current_gen
    }

    pub fn reset(&mut self) -> () {
        self.current_gen = 0;
        self.grid = self.init_state.clone();
    }
}
