use crossterm::style::{style, Attribute, Color};
use std::collections::HashMap;

mod automaton;
mod commands;
mod conway;
mod terminal_ui;

use conway::{GameOfLife};
use terminal_ui::TerminalUI;

fn main() -> () {
    let mut printer = HashMap::new();
    printer.insert(GameOfLife::Dead, style('·').with(Color::Grey));
    printer.insert(
        GameOfLife::Alive,
        style('#').with(Color::Green).attribute(Attribute::Bold),
    );

    let mut term_ui = TerminalUI::new(conway::conway_canon(), printer);
    term_ui.cmd_interpreter().unwrap();
}
