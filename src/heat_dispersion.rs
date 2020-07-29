// External libraries
use crossterm::style::{style, Attribute, Color, StyledContent};

// CELL
use crate::automaton::*;
use crate::grid::{Dimensions, Grid, GridView, Position, MOORE_NEIGHBORHOOD};

impl AutomatonCell for u8 {}

impl CPUCell for u8 {
    fn update_cell<'a>(grid: &GridView<'a, Self>) -> Self {
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

impl TermDrawableAutomaton for u8 {
    fn style(&self) -> StyledContent<char> {
        style('#')
            .with(Color::Rgb {
                r: *self,
                g: 0u8,
                b: 255 - *self,
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
