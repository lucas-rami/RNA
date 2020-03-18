use crossterm::{
    style::{style, Attribute, Color},
};
use std::collections::HashMap;

use std::{thread, time};

mod automaton;
mod conway;
mod terminal_ui;

use conway::{conway_canon, GameOfLife};
use terminal_ui::{Operation as TermOP, TerminalUI};

fn main() -> () {
    let mut printer = HashMap::new();
    printer.insert(GameOfLife::Dead, style('·'));
    printer.insert(
        GameOfLife::Alive,
        style('#').with(Color::Blue).attribute(Attribute::Bold),
    );

    let mut automaton = conway_canon();
    let mut term_ui = TerminalUI::new();
    term_ui.perform(TermOP::BindAutomaton(&automaton, printer));
    term_ui.cmd_interpreter(&mut automaton).unwrap();

    // conway.perform(Operation::Goto(10));
    // term_ui.perform(TermOP::SetState(&conway));
    // term_ui.perform(TermOP::NotifyEvolution(50));

    // for _x in 0..50 {
    //     thread::sleep(time::Duration::from_millis(200));
    //     conway.perform(Operation::Step);
    //     term_ui.perform(TermOP::SetState(&conway));
    // }
    }

