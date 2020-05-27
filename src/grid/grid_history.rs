// Standard library
use std::collections::HashMap;
use std::fmt::Debug;

// CELL
use super::{Grid, Position, PositionIterator};
use crate::advanced_channels::{MailType, SlaveEndpoint};

pub struct GridHistory<T: Copy + Debug + Default + Eq + PartialEq> {
    diffs: Vec<GridDiff<T>>,
    checkpoints: Vec<Grid<T>>,
    f_check: usize,
    last: Grid<T>,
}

impl<T: Copy + Debug + Default + Eq + PartialEq> GridHistory<T> {
    pub fn new(initial_grid: &Grid<T>, f_check: usize) -> Self {
        Self {
            diffs: vec![],
            checkpoints: vec![initial_grid.clone()],
            f_check,
            last: initial_grid.clone(),
        }
    }

    pub fn push(&mut self, grid: Grid<T>) {
        self.diffs.push(GridDiff::new(&self.last, &grid));
        if self.f_check != 0 && self.diffs.len() % self.f_check == 0 {
            self.checkpoints.push(grid.clone());
        }
        self.last = grid;
    }

    pub fn get_gen(&self, gen: usize) -> Option<Grid<T>> {
        if self.diffs.len() < gen {
            // We don't have that generation
            None
        } else {
            // We have the generation
            if self.f_check != 0 {
                let idx = gen / self.f_check;
                let shift = gen % self.f_check;

                // Accumulate differences between reference grid and target generation
                let stacked_diffs = GridDiff::stack(&self.diffs[(gen - shift)..gen]);

                // Apply modifications on reference grid
                let mut grid = self.checkpoints[idx as usize].clone();
                grid.apply_diffs(stacked_diffs);
                Some(grid)
            } else {
                // Accumulate differences between initial grid and target generation
                let stacked_diffs = GridDiff::stack(&self.diffs[0..gen]);
                let mut grid = self.checkpoints[0].clone();
                grid.apply_diffs(stacked_diffs);
                Some(grid)
            }
        }
    }

    pub fn diff(&self, base_gen: usize, target_gen: usize) -> Option<GridDiff<T>> {
        if target_gen < base_gen {
            panic!("Base generation should be smaller than target generation.");
        }
        if self.diffs.len() < target_gen {
            None
        } else {
            Some(GridDiff::stack(&self.diffs[base_gen..target_gen]))
        }
    }

    pub fn dispatch(mut self, endpoint: SlaveEndpoint<Option<Grid<T>>, GridHistoryOP<T>>) {
        loop {
            match endpoint.wait_for_mail() {
                MailType::SimpleMsg(msg) => match msg {
                    GridHistoryOP::Push(grid) => self.push(grid),
                    _ => panic!(ERR_INCOMPATIBLE_MAIL_TYPE),
                },
                MailType::ResponseRequired(req) => match req.get_request() {
                    GridHistoryOP::GetGen { gen, blocking } => match self.get_gen(*gen) {
                        Some(grid) => {
                            req.respond(Some(grid));
                        }
                        None => {
                            if *blocking {
                                loop {
                                    match endpoint.wait_for_simple_msg() {
                                        GridHistoryOP::Push(grid) => {
                                            self.push(grid);
                                            if let Some(response_grid) = self.get_gen(*gen) {
                                                req.respond(Some(response_grid));
                                                break;
                                            }
                                        }
                                        _ => panic!(ERR_INCOMPATIBLE_MAIL_TYPE),
                                    }
                                }
                            } else {
                                req.respond(None);
                            }
                        }
                    },
                    _ => panic!(ERR_INCOMPATIBLE_MAIL_TYPE),
                },
                MailType::DeadChannel => break,
            }
        }
    }
}

#[derive(Debug)]
pub struct GridDiff<T: Copy + Default + PartialEq> {
    diffs: HashMap<Position, T>,
}

impl<T: Copy + Default + Eq + PartialEq> GridDiff<T> {
    pub fn new(prev_grid: &Grid<T>, next_grid: &Grid<T>) -> Self {
        let dim = prev_grid.dim();
        if dim != next_grid.dim() {
            panic!("Both grids should be the same dimensions!")
        }

        let mut diffs = HashMap::new();
        for (pos, (prev, next)) in
            PositionIterator::new(*dim).zip(prev_grid.iter().zip(next_grid.iter()))
        {
            if prev != next {
                diffs.insert(pos, *next);
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

pub enum GridHistoryOP<T: Copy + Default + Eq + PartialEq> {
    Push(Grid<T>),
    GetGen { gen: usize, blocking: bool },
}

const ERR_INCOMPATIBLE_MAIL_TYPE: &str =
    "The received GridHistoryOP is incompatible with the MailType it's included in.";
