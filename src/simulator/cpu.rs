// CELL
use super::{CellularAutomaton, Simulator};
use crate::grid::{Dimensions, Grid, Position};

pub struct CPUSimulator<A: CellularAutomaton> {
    name: String,
    automaton: A,
    grid: Grid<A::State>,
    current_gen: u64,
}

impl<A: CellularAutomaton> CPUSimulator<A> {
    pub fn new(name: &str, automaton: A, grid: &Grid<A::State>) -> Self {
        Self {
            name: String::from(name),
            automaton,
            grid: grid.clone(),
            current_gen: 0,
        }
    }
}

impl<A: CellularAutomaton> Simulator<A> for CPUSimulator<A> {
    fn run(&mut self, nb_gens: u64) -> () {
        for _ in 0..nb_gens {
            let dim = self.grid.dim();
            let mut new_data = Vec::with_capacity(dim.size() as usize);
            for row in 0..dim.height() {
                for col in 0..dim.width() {
                    let pos = Position::new(col, row);
                    let view = self.grid.view(pos.clone());
                    let new_state = self.automaton.update_cpu(&view);
                    new_data.push(new_state);
                }
            }
            self.grid.switch_data(new_data);
        }
        self.current_gen += nb_gens
    }

    fn automaton(&self) -> &A {
        &self.automaton
    }

    fn cell(&self, pos: Position) -> A::State {
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
