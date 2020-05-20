// Standard library
use std::collections::HashMap;

// CELL
use super::{Grid, Position};

pub struct GridHistory<T: Copy + Default + Eq + PartialEq> {
    diffs: Vec<GridDiff<T>>,
    checkpoints: Vec<Grid<T>>,
    f_check: u64,
    last: Grid<T>,
}

impl<T: Copy + Default + Eq + PartialEq> GridHistory<T> {
    pub fn new(initial_grid: Grid<T>, f_check: u64) -> Self {
        Self {
            diffs: vec![],
            checkpoints: vec![initial_grid.clone()],
            f_check,
            last: initial_grid,
        }
    }

    pub fn push(&mut self, grid: Grid<T>) {
        self.diffs.push(GridDiff::new(&self.last, &grid));
        if self.f_check != 0 && self.diffs.len() as u64 % self.f_check == 0 {
            self.checkpoints.push(grid.clone());
        }
        self.last = grid;
    }

    pub fn gen(&self, gen: u64) -> Option<Grid<T>> {
        let max_gen = self.diffs.len() as u64;
        if max_gen as u64 + 1 < gen {
            // We don't have that generation
            None
        } else {
            // We have the generation
            if self.f_check != 0 {
                let idx = gen / self.f_check;
                let shift = gen % self.f_check;

                // Accumulate differences between reference grid and target generation
                let low_idx = (gen - shift) as usize;
                let high_idx = gen as usize;
                let stacked_diffs = GridDiff::stack(&self.diffs[low_idx..high_idx]);

                // Apply modifications on reference grid
                let mut grid = self.checkpoints[idx as usize].clone();
                grid.apply_diffs(stacked_diffs);
                Some(grid)
            } else {
                // Accumulate differences between initial grid and target generation
                let stacked_diffs = GridDiff::stack(&self.diffs[0..(gen as usize)]);
                let mut grid = self.checkpoints[0].clone();
                grid.apply_diffs(stacked_diffs);
                Some(grid)
            }
        }
    }
}

pub struct GridDiff<T: Copy + Default + Eq + PartialEq> {
    diffs: HashMap<Position, T>,
}

impl<T: Copy + Default + Eq + PartialEq> GridDiff<T> {
    pub fn new(prev_grid: &Grid<T>, next_grid: &Grid<T>) -> Self {
        let dim = prev_grid.dim();
        if dim != next_grid.dim() {
            panic!("Both grids should be the same dimensions!")
        }

        let mut x = 0;
        let mut y = 0;

        let mut diffs = HashMap::new();
        for (prev, next) in prev_grid.iter().zip(next_grid.iter()) {
            // Check if there is a difference
            if prev != next {
                diffs.insert(Position::new(x, y), *next);
            }

            // Update position
            if x == dim.width - 1 {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }

        Self { diffs }
    }

    pub fn merge_with(&mut self, other: &Self) {
        for (pos, new_cell) in other.diffs.iter() {
            match self.diffs.get_mut(pos) {
                Some(old_cell) => *old_cell = *new_cell,
                None => {
                    self.diffs.insert(*pos, *new_cell);
                }
            }
        }
    }

    pub fn stack(diffs_list: &[Self]) -> Self {
        if diffs_list.len() == 0 {
            Self::default()
        } else {
            let mut stacked_diffs = Self::default();
            for diffs in diffs_list.iter() {
                stacked_diffs.merge_with(diffs);
            }
            stacked_diffs
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Position, &T)> {
        self.diffs.iter()
    }
}

impl<T: Copy + Default + Eq + PartialEq> Default for GridDiff<T> {
    fn default() -> Self {
        Self {
            diffs: HashMap::new(),
        }
    }
}
