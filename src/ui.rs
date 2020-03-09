use crate::automaton::{Cells, CellularAutomaton};
use crossterm::{
    cursor, queue,
    style::{self, PrintStyledContent, StyledContent},
    terminal,
};
use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};

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

    pub fn new() -> Self {
        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = TerminalUI::create_modules(size);
        let mut ui = TerminalUI {
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
        let modules = TerminalUI::create_modules(size);
        self.size = size;
        self.auto_mod = modules.0;
        self.info_mod = modules.1;

        self.clear_and_draw_all();
    }

    pub fn set_auto_render_pos(&mut self, pos: (usize, usize)) -> () {
        self.auto_render_pos = pos;
    }

    fn create_modules(size: Position) -> (Module, Module) {
        let height_automaton = size.1 - HEIGHT_INFO - 2;
        let auto_mod = Module::new(
            (0, 0),
            (size.0, height_automaton),
            String::from("Automaton"),
        );
        let info_mod = Module::new(
            (0, height_automaton),
            (size.0, HEIGHT_INFO),
            String::from("Information"),
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

struct Module {
    pos: Position,
    size: Position,
    title: String,
}

impl Module {
    fn new(pos: Position, size: Position, title: String) -> Self {
        if size.0 < 3 || size.1 < 3 {
            panic!("Module size must be at least 3x3.")
        }
        Module { pos, size, title }
    }

    fn clear_content(&self, stdout: &mut Stdout) -> () {
        let content_pos = self.get_render_pos();
        let content_size = self.get_render_size();

        let empty_line = std::iter::repeat(" ")
            .take(content_size.0 as usize)
            .collect::<String>();

        for x in 0..content_size.1 {
            queue!(
                stdout,
                cursor::MoveTo(content_pos.0, content_pos.1 + x),
                style::Print(empty_line.clone())
            )
            .expect("Failed to clear module content.")
        }
    }

    fn draw(&self, stdout: &mut Stdout) -> () {
        let err_msg = "Failed to draw module.";

        // Draw top line
        queue!(
            stdout,
            cursor::MoveTo(self.pos.0, self.pos.1),
            style::Print("┌")
        )
        .expect(err_msg);
        for _ in 1..(self.size.0 - 1) {
            queue!(stdout, style::Print('─')).expect(err_msg);
        }
        queue!(stdout, style::Print('┐')).expect(err_msg);

        // Draw vertical lines
        for row in (self.pos.1 + 1)..(self.pos.1 + self.size.1 - 1) {
            queue!(
                stdout,
                cursor::MoveTo(self.pos.0, row),
                style::Print('│'),
                cursor::MoveTo(self.pos.0 + self.size.0 - 1, row),
                style::Print('│')
            )
            .expect(err_msg);
        }

        // Draw bottom line
        queue!(
            stdout,
            cursor::MoveTo(self.pos.0, self.pos.1 + self.size.1 - 1),
            style::Print('└')
        )
        .expect(err_msg);
        for _ in 1..(self.size.0 - 1) {
            queue!(stdout, style::Print('─')).expect(err_msg);
        }
        queue!(stdout, style::Print('┘')).expect(err_msg);

        // Draw title
        let title_len = self.title.chars().count();
        if title_len + 2 <= (self.size.0 - 4) as usize {
            queue!(
                stdout,
                cursor::MoveTo(self.pos.0 + 2, self.pos.1),
                style::Print(format!(" {} ", self.title))
            )
            .expect(err_msg);
        }
    }

    fn get_render_pos(&self) -> Position {
        (self.pos.0 + 1, self.pos.1 + 1)
    }

    fn get_render_size(&self) -> Position {
        (self.size.0 - 2, self.size.1 - 2)
    }
}
