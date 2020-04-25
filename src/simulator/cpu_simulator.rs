// CELL
use super::grid::{Dimensions, Grid, Position};
use super::{CellularAutomaton, Simulator};

pub struct CPUSimulator<S: Copy, C: CellularAutomaton<S>> {
    name: String,
    automaton: C,
    grid: Grid<S>,
    current_gen: u64,
}

impl<S: Copy, C: CellularAutomaton<S>> CPUSimulator<S, C> {
    pub fn new(name: &str, automaton: C, grid: &Grid<S>) -> Self {
        Self {
            name: String::from(name),
            automaton,
            grid: grid.clone(),
            current_gen: 0,
        }
    }
}

impl<S: Copy, C: CellularAutomaton<S>> Simulator<S, C> for CPUSimulator<S, C> {
    fn run(&mut self, nb_gens: u64) -> () {
        for _ in 0..nb_gens {
            let dim = self.grid.dim();
            let mut new_grid = Grid::new(dim.clone(), &self.automaton.default());
            for row in 0..dim.nb_rows {
                for col in 0..dim.nb_cols {
                    let pos = Position::new(col, row);
                    let view = self.grid.view(pos.clone());
                    let new_state = self.automaton.update_cpu(&view);
                    new_grid.set(&pos, new_state);
                }
            }
            self.grid = new_grid;
        }
        self.current_gen += nb_gens
    }

    fn automaton(&self) -> &C {
        &self.automaton
    }

    fn cell(&self, pos: &Position) -> &S {
        self.grid.get(pos)
    }

    fn size(&self) -> &Dimensions {
        self.grid.dim()
    }

    fn name(&self) -> &str {
        &self.name[..]
    }

    fn current_gen(&self) -> u64 {
        self.current_gen
    }
}
