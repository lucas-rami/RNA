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
                if ((self.f_check >> 1) < shift) && (idx + 1) * self.f_check <= max_gen as u64 {
                    // Accumulate differences betwween reference grid and target generation
                    let low_idx = (gen + 1) as usize;
                    let high_idx = (gen + self.f_check - shift) as usize;
                    let stacked_diffs = GridDiff::stack(&self.diffs[low_idx..high_idx], true);

                    // Apply modifications on reference grid
                    let mut grid = self.checkpoints[idx as usize + 1].clone();
                    stacked_diffs.apply_on(&mut grid, true);
                    Some(grid)
                } else {
                    // Accumulate differences betwween reference grid and target generation
                    let low_idx = (gen - shift) as usize;
                    let high_idx = gen as usize;
                    let stacked_diffs = GridDiff::stack(&self.diffs[low_idx..high_idx], false);

                    // Apply modifications on reference grid
                    let mut grid = self.checkpoints[idx as usize].clone();
                    stacked_diffs.apply_on(&mut grid, false);
                    Some(grid)
                }
            } else {
                if max_gen - gen < gen {
                    // Accumulate differences betwween last grid and target generation
                    let stacked_diffs = GridDiff::stack(&self.diffs[(gen as usize)..], true);
                    let mut grid = self.last.clone();
                    stacked_diffs.apply_on(&mut grid, true);
                    Some(grid)
                } else {
                    // Accumulate differences betwween initial grid and target generation
                    let stacked_diffs = GridDiff::stack(&self.diffs[0..(gen as usize)], false);
                    let mut grid = self.checkpoints[0].clone();
                    stacked_diffs.apply_on(&mut grid, false);
                    Some(grid)
                }
            }
        }
    }

    pub fn diff(&self, base_gen: u64, target_gen: u64) -> Option<GridDiff<T>> {
        let max_gen = self.diffs.len() as u64;
        if max_gen < base_gen || target_gen < base_gen {
            None
        } else {
            let rev = target_gen < base_gen;
            Some(GridDiff::stack(
                &self.diffs[(base_gen as usize)..(target_gen as usize)],
                rev,
            ))
        }
    }
}

pub struct GridDiff<T: Copy + Default + Eq + PartialEq> {
    diffs: HashMap<Position, Diff<T>>,
}

impl<T: Copy + Default + Eq + PartialEq> GridDiff<T> {
    fn new(prev_grid: &Grid<T>, next_grid: &Grid<T>) -> Self {
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
                diffs.insert(
                    Position::new(x, y),
                    Diff {
                        prev: *prev,
                        next: *next,
                    },
                );
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

    fn apply_on(&self, grid: &mut Grid<T>, rev: bool) {
        let iter = self.diffs.iter();
        if rev {
            for (pos, diff) in iter {
                grid.set(*pos, diff.prev);
            }
        } else {
            for (pos, diff) in iter {
                grid.set(*pos, diff.next);
            }
        }
    }

    fn merge_with(&mut self, other: &Self, rev: bool) {
        let iter = other.diffs.iter();
        if rev {
            for (pos, diff) in iter {
                match self.diffs.get_mut(pos) {
                    Some(old_diff) => old_diff.prev = diff.prev,
                    None => {
                        self.diffs.insert(*pos, *diff);
                    }
                }
            }
        } else {
            for (pos, diff) in iter {
                match self.diffs.get_mut(pos) {
                    Some(old_diff) => old_diff.next = diff.next,
                    None => {
                        self.diffs.insert(*pos, *diff);
                    }
                }
            }
        }
    }

    fn stack(diffs_list: &[Self], rev: bool) -> Self {
        if diffs_list.len() == 0 {
            Self::default()
        } else {
            let mut stacked_diffs = Self::default();
            if rev {
                for diffs in diffs_list.iter().rev() {
                    stacked_diffs.merge_with(diffs, rev);
                }
            } else {
                for diffs in diffs_list.iter() {
                    stacked_diffs.merge_with(diffs, rev);
                }
            }

            stacked_diffs
        }
    }
}

impl<T: Copy + Default + Eq + PartialEq> Default for GridDiff<T> {
    fn default() -> Self {
        Self {
            diffs: HashMap::new(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Diff<T: Copy> {
    prev: T,
    next: T,
}
