use crate::automaton::{CellularAutomaton, Cells};
use crossterm::{cursor, queue, style, terminal};
use std::io::{stdout, Stdout, Write};

const HEIGHT_INFO: u16 = 10;

pub struct TerminalUI {
    stdout: Stdout,
    size: (u16, u16),
    auto_mod: Module,
    info_mod: Module,
}

impl TerminalUI {
    pub fn draw_automaton<C: Cells>(&self, automaton: CellularAutomaton<C>) -> () {}

    pub fn new() -> Self {
        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = TerminalUI::create_modules(size);
        let mut ui = TerminalUI {
            stdout: stdout(),
            size,
            auto_mod: modules.0,
            info_mod: modules.1,
        };

        ui.clear_and_draw_all();
        ui
    }

    pub fn resize(&mut self, size: (u16, u16)) -> () {
        let modules = TerminalUI::create_modules(size);
        self.size = size;
        self.auto_mod = modules.0;
        self.info_mod = modules.1;

        self.clear_and_draw_all();
    }

    fn create_modules(size: (u16, u16)) -> (Module, Module) {
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
    pos: (u16, u16),
    size: (u16, u16),
    title: String,
}

impl Module {
    fn new(pos: (u16, u16), size: (u16, u16), title: String) -> Self {
        Module { pos, size, title }
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
}
