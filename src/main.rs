use crossterm::style::{style, Attribute, Color};
use std::collections::HashMap;

mod automaton;
mod commands;
mod conway;
mod terminal_ui;

use conway::{conway_canon, GameOfLife};
use terminal_ui::TerminalUI;

fn main() -> () {
    let mut printer = HashMap::new();
    printer.insert(GameOfLife::Dead, style('·'));
    printer.insert(
        GameOfLife::Alive,
        style('#').with(Color::Blue).attribute(Attribute::Bold),
    );

    let mut automaton = conway_canon();
    let mut term_ui = TerminalUI::new();
    term_ui.bind_automaton(automaton, printer);
    term_ui.cmd_interpreter().unwrap();
}
