pub trait Cells: Clone {
    fn default() -> Self;
    fn update_cell(grid: &CellularAutomaton<Self>, row: usize, col: usize) -> Self;
}

pub struct CellularAutomaton<C: Cells> {
    nb_rows: usize,
    nb_cols: usize,
    state: State,
    current_gen: u64,
    initial_state: Vec<C>,
    automaton: Vec<C>,
}

impl<C: Cells> CellularAutomaton<C> {
    pub fn new(nb_rows: usize, nb_cols: usize) -> CellularAutomaton<C> {
        CellularAutomaton {
            nb_rows,
            nb_cols,
            state: State::Building,
            current_gen: 0,
            initial_state: vec![C::default(); nb_rows * nb_cols],
            automaton: vec![],
        }
    }

    pub fn perform(&mut self, op: Operation<C>) -> () {
        match self.state {
            State::Building => match op {
                Operation::SetCell(x, y, new_state) => self.set_cell(x, y, new_state),
                Operation::LockInitialState => self.lock_init_state(),
                _ => panic!("Unsupported operation."),
            },
            State::Ready => match op {
                Operation::Reset => self.reset(),
                Operation::Run(nb_gens) => self.run(nb_gens),
                Operation::Step => self.perform(Operation::Run(1)),
                Operation::Goto(gen_number) if gen_number >= self.current_gen => {
                    self.perform(Operation::Run(gen_number - self.current_gen))
                }
                _ => panic!("Unsupported operation."),
            },
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &C {
        if self.nb_rows <= row || self.nb_cols <= col {
            panic!("Invalid grid index.")
        }
        &self.automaton[row * self.nb_cols + col]
    }

    pub fn neighbor(&self, row: usize, col: usize, direction: &Neighbor) -> C {
        match direction {
            Neighbor::Top => {
                if row == 0 {
                    C::default()
                } else {
                    self.get_cell(row - 1, col).clone()
                }
            }
            Neighbor::TopRight => {
                if row == 0 || col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get_cell(row - 1, col + 1).clone()
                }
            }
            Neighbor::Right => {
                if col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get_cell(row, col + 1).clone()
                }
            }
            Neighbor::BottomRight => {
                if row == self.nb_rows - 1 || col == self.nb_cols - 1 {
                    C::default()
                } else {
                    self.get_cell(row + 1, col + 1).clone()
                }
            }
            Neighbor::Bottom => {
                if row == self.nb_rows - 1 {
                    C::default()
                } else {
                    self.get_cell(row + 1, col).clone()
                }
            }
            Neighbor::BottomLeft => {
                if row == self.nb_rows - 1 || col == 0 {
                    C::default()
                } else {
                    self.get_cell(row + 1, col - 1).clone()
                }
            }
            Neighbor::Left => {
                if col == 0 {
                    C::default()
                } else {
                    self.get_cell(row, col - 1).clone()
                }
            }
            Neighbor::TopLeft => {
                if row == 0 || col == 0 {
                    C::default()
                } else {
                    self.get_cell(row - 1, col - 1).clone()
                }
            }
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.nb_cols, self.nb_rows)
    }

    pub fn current_gen(&self) -> u64 {
        self.current_gen
    }

    pub fn is_initialized(&self) -> bool {
        match self.state {
            State::Ready => true,
            _ => false,
        }
    }

    fn set_cell(&mut self, x: usize, y: usize, new_state: C) -> () {
        if self.nb_cols <= x || self.nb_rows <= y {
            panic!("Cell index is invalid.")
        }
        self.initial_state[y * self.nb_cols + x] = new_state
    }

    fn lock_init_state(&mut self) -> () {
        self.state = State::Ready;
        self.automaton = self.initial_state.clone();
    }

    fn reset(&mut self) -> () {
        self.current_gen = 0;
        self.automaton = self.initial_state.clone();
    }

    fn run(&mut self, nb_gens: u64) -> () {
        for i in 0..nb_gens {
            let mut new_automaton = vec![];
            for row in 0..self.nb_rows {
                for col in 0..self.nb_cols {
                    new_automaton.push(C::update_cell(&self, row, col));
                }
            }
            self.automaton = new_automaton;
        }
        self.current_gen += nb_gens
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

pub enum Operation<C: Cells> {
    // Valid in "Building" state
    SetCell(usize, usize, C),
    LockInitialState,
    // Valid in "Ready" state
    Reset,
    Step,
    Run(u64),
    Goto(u64),
}

enum State {
    Building,
    Ready,
}
