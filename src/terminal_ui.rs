use crate::automaton::{Cells, CellularAutomaton};
use crossterm::{
    cursor, queue,
    style::{self, style, Attribute, PrintStyledContent, StyledContent},
    terminal,
};
use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};

mod module;
mod styled_text;

use module::Module;
use styled_text::StyledText;

type Position = (u16, u16);

const HEIGHT_INFO: u16 = 10;

pub struct TerminalUI {
    stdout: Stdout,
    size: Position,
    auto_mod: Module,
    info_mod: Module,
    auto_render_pos: (usize, usize),
}

impl TerminalUI {
    pub fn new() -> Self {
        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = Self::create_modules(size);
        let mut ui = Self {
            stdout: stdout(),
            size,
            auto_mod: modules.0,
            info_mod: modules.1,
            auto_render_pos: (0, 0),
        };

        ui.clear_and_draw_all();
        ui
    }

    pub fn resize(&mut self, size: Position) -> () {
        let modules = Self::create_modules(size);
        self.size = size;
        self.auto_mod = modules.0;
        self.info_mod = modules.1;

        self.clear_and_draw_all();
    }

    pub fn draw_automaton<C: Cells>(
        &mut self,
        automaton: &CellularAutomaton<C>,
        style: &HashMap<C, StyledContent<char>>,
    ) -> () {
        // Get maximum render size and convert to (usize, usize)
        let max_render_size = self.auto_mod.get_render_size();
        let max_render_size = (max_render_size.0 as usize, max_render_size.1 as usize);

        // Determine real render size
        let auto_size = automaton.size();
        let mut render_size = (
            auto_size.0 - self.auto_render_pos.0,
            auto_size.1 - self.auto_render_pos.1,
        );
        if render_size.0 > max_render_size.0 {
            render_size.0 = max_render_size.0;
        }
        if render_size.1 > max_render_size.1 {
            render_size.1 = max_render_size.1;
        }

        // Clear module content and redraw over it
        self.auto_mod.clear_content(&mut self.stdout);

        let render_pos = self.auto_mod.get_render_pos();
        let mut row = self.auto_render_pos.1;
        for y in 0..render_size.1 {
            queue!(
                self.stdout,
                cursor::MoveTo(render_pos.0, render_pos.1 + (y as u16))
            )
            .expect("Failed to move cursor.");
            for x in 0..render_size.0 {
                let c = match style.get(automaton.get_cell(row, self.auto_render_pos.0 + x)) {
                    Some(repr) => repr.clone(),
                    None => style::style('?'),
                };
                queue!(self.stdout, PrintStyledContent(c)).expect("Failed to display automaton");
            }
            // Next row
            row += 1
        }

        // @TODO: update info module

        // Flush
        self.cursor_to_command();
        self.flush();
    }

    pub fn set_auto_render_pos(&mut self, pos: (usize, usize)) -> () {
        self.auto_render_pos = pos;
    }

    fn create_modules(size: Position) -> (Module, Module) {
        let height_automaton = size.1 - HEIGHT_INFO - 2;
        let auto_mod = Module::new(
            StyledText::from(vec![style(String::from("Automaton"))]),
            (0, 0),
            (size.0, height_automaton),
        );
        let info_mod = Module::new(
            StyledText::from(vec![
                style(String::from("Information")).attribute(Attribute::Italic)
            ]),
            (0, height_automaton),
            (size.0, HEIGHT_INFO),
        );
        (auto_mod, info_mod)
    }

    fn cursor_to_command(&mut self) -> () {
        queue!(
            self.stdout,
            cursor::MoveTo(0, self.size.1 - 1),
            style::Print("> ")
        )
        .expect("Failed to move cursor to command line.");
    }

    fn clear_and_draw_all(&mut self) -> () {
        queue!(self.stdout, terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");
        self.draw_modules();
        self.cursor_to_command();
        self.flush();
    }

    fn draw_modules(&mut self) -> () {
        self.auto_mod.draw(&mut self.stdout);
        self.info_mod.draw(&mut self.stdout);
    }

    fn flush(&mut self) -> () {
        self.stdout.flush().expect("Failed to flush stdout.");
    }
}
