use crossterm::{
    style::{style, Attribute, Color},
    Result,
};
use std::collections::HashMap;

use std::io::stdin;
use std::{thread, time};

mod automaton;
mod conway;
mod ui;

use automaton::CellularAutomaton;
use conway::{GameOfLife, conway_canon};


fn main() -> Result<()> {
    let mut display = HashMap::new();
    display.insert(GameOfLife::Dead, style('·'));
    display.insert(
        GameOfLife::Alive,
        style('#').with(Color::Blue).attribute(Attribute::Bold),
    );

    // let mut conway = CellularAutomaton::<GameOfLife>::new(20, 50);
    // conway.set_cell(3, 4, GameOfLife::Alive);
    // conway.set_cell(3, 5, GameOfLife::Alive);
    // conway.set_cell(3, 6, GameOfLife::Alive);

    let mut conway = conway_canon();
    let mut term_ui = ui::TerminalUI::new();
    term_ui.draw_automaton(&conway, &display);

    for _x in 0..1000 {
        term_ui.draw_automaton(&conway, &display);
        thread::sleep(time::Duration::from_millis(100));
        conway.run();
    }

    let mut tmp = String::new();
    stdin().read_line(&mut tmp)?;

    Ok(())
}
