// External libraries
use cascade::cascade;
use crossterm::style::{style, Attribute, Color, StyledContent};

// CELL
use crate::grid::{Dimensions, Grid, GridView, Position, MOORE_NEIGHBORHOOD};
use crate::simulator::{CPUComputableAutomaton, CellType, CellularAutomaton};
use crate::terminal_ui::TermDrawableAutomaton;

pub struct HeatDispersion {
    name: &'static str,
}

impl HeatDispersion {
    pub fn new() -> Self {
        Self {
            name: "Heat Dispersion",
        }
    }
}

impl CellularAutomaton for HeatDispersion {
    type Cell = u8;

    fn name(&self) -> &str {
        self.name
    }
}

impl CellType for u8 {}

impl CPUComputableAutomaton for HeatDispersion {
    fn update_cell<'a>(grid: &GridView<'a, Self::Cell>) -> Self::Cell {
        let total_heat = grid
            .get_relative_mul(&MOORE_NEIGHBORHOOD)
            .iter()
            .fold(0u32, |acc, cell| acc + (**cell as u32));

        let average_heat = total_heat / (MOORE_NEIGHBORHOOD.len() as u32);
        if average_heat > 255 {
            255
        } else {
            average_heat as u8
        }
    }
}

impl TermDrawableAutomaton for HeatDispersion {
    fn style(&self, state: &Self::Cell) -> StyledContent<char> {
        style('#')
            .with(Color::Rgb {
                r: *state,
                g: 0u8,
                b: 255 - *state,
            })
            .attribute(Attribute::Bold)
    }
}

pub fn basic() -> Grid<u8> {
    let mut grid = Grid::new(Dimensions::new(40, 40));
    for x in 10..30 {
        if x <= 15 || 25 <= x { 
            for y in 10..30 {
                grid.set(Position::new(x, y), 255);
            }
        }
    }
    grid
}
