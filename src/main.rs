use crossterm::{
    cursor, execute, queue,
    style::{style, Attribute, Color, PrintStyledContent, StyledContent},
    terminal, Result,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{stdin, stdout, Write};
use std::{thread, time};

mod automaton;
mod ui;
mod conway;

use automaton::CellularAutomaton;
use conway::GameOfLife;


fn main() -> Result<()> {
    let mut display = HashMap::new();
    display.insert(GameOfLife::Dead, style('·'));
    display.insert(
        GameOfLife::Alive,
        style('#').with(Color::Blue).attribute(Attribute::Bold),
    );

    let mut conway = CellularAutomaton::<GameOfLife>::new(10, 20);
    conway.set_cell(3, 4, GameOfLife::Alive);
    conway.set_cell(3, 5, GameOfLife::Alive);
    conway.set_cell(3, 6, GameOfLife::Alive);
    
    let mut term_ui = ui::TerminalUI::new();

    let mut tmp = String::new();
    stdin().read_line(&mut tmp).expect("Failed to read line");

    // execute!(stdout(), terminal::SetSize(100, 100))?;
    // thread::sleep(time::Duration::from_millis(5000));
    Ok(())
}
