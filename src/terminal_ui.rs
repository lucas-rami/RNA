use crate::commands::Command;
use crate::simulator::{
    automaton::{CellularAutomaton, TermDrawable},
    grid::Position,
    Simulator,
};
use crossterm::{
    cursor,
    event::{Event, KeyCode},
    queue,
    style::{style, Attribute, Print, PrintStyledContent},
    terminal,
};
use std::io::{stdout, Write};
use std::{thread, time};

mod module;
mod styled_text;

use module::Module;
use styled_text::StyledText;

type Size = (u16, u16);

const HEIGHT_INFO: u16 = 10;
const RUN: &str = "run";
const GOTO: &str = "goto";
const VIEW: &str = "view";

pub struct TerminalUI<C: CellularAutomaton + TermDrawable> {
    size: Size,
    auto_mod: Module,
    info_mod: Module,
    view: (usize, usize),
    commands: Vec<Command>,
    simulator: Simulator<C>,
}

impl<C: CellularAutomaton + TermDrawable> TerminalUI<C> {
    pub fn new(simulator: Simulator<C>) -> Self {
        // Clear terminal
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");

        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = Self::create_modules(size);
        let mut ui = Self {
            size,
            auto_mod: modules.0,
            info_mod: modules.1,
            view: (0, 0),
            commands: vec![
                Command::new(RUN, vec!["nb_gens"]),
                Command::new(GOTO, vec!["target_gen"]),
                Command::new(VIEW, vec!["x", "y"]),
            ],
            simulator,
        };

        // Set simulator title and draw initial state
        let title = StyledText::from(vec![style(String::from(ui.simulator.get_name()))]);
        ui.auto_mod.set_title(title);
        ui.draw_automaton();
        ui
    }

    pub fn cmd_interpreter(&mut self) -> crossterm::Result<()> {
        // Ensure cursor is on command line
        self.cursor_to_command();
        let base_pos = cursor::position()?;
        let max_len = self.size.0 - base_pos.0;
        let mut output = stdout();
        // History
        let mut history: Vec<Vec<char>> = vec![];

        // Status for current command
        let mut cmd = vec![];
        let mut mem_cmd = vec![];
        let mut line_pos: usize = 0;
        let mut history_idx: usize = 0;

        loop {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char(c) => {
                            if cmd.len() < max_len as usize {
                                // Update command
                                cmd.insert(line_pos, c);

                                // Display new string
                                queue!(
                                    output,
                                    cursor::MoveTo(base_pos.0 + (line_pos as u16), base_pos.1),
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                    Print::<String>((&cmd[line_pos..]).iter().collect()),
                                    cursor::MoveTo(base_pos.0 + (line_pos as u16) + 1, base_pos.1),
                                )?;

                                line_pos += 1;
                            }
                        }
                        KeyCode::Left => {
                            if 0 < line_pos {
                                queue!(output, cursor::MoveLeft(1))?;
                                line_pos -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if line_pos < cmd.len() {
                                queue!(output, cursor::MoveRight(1))?;
                                line_pos += 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if 0 < line_pos {
                                // Update command
                                cmd.remove(line_pos - 1);
                                line_pos -= 1;

                                // Display new string
                                queue!(
                                    output,
                                    cursor::MoveTo(base_pos.0 + (line_pos as u16), base_pos.1),
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                    Print::<String>((&cmd[line_pos..]).iter().collect()),
                                    cursor::MoveTo(base_pos.0 + (line_pos as u16), base_pos.1),
                                )?;
                            }
                        }
                        KeyCode::Delete => {
                            if line_pos < cmd.len() {
                                // Update command
                                cmd.remove(line_pos);

                                // Display new string
                                queue!(
                                    output,
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                    Print::<String>((&cmd[line_pos..]).iter().collect()),
                                    cursor::MoveTo(base_pos.0 + (line_pos as u16), base_pos.1),
                                )?;
                            }
                        }
                        KeyCode::Up => {
                            if 0 < history_idx {
                                if history_idx == history.len() {
                                    mem_cmd = cmd;
                                }
                                history_idx -= 1;
                                cmd = history[history_idx].clone();
                                line_pos = cmd.len();
                                queue!(
                                    output,
                                    cursor::MoveTo(base_pos.0, base_pos.1),
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                    Print::<String>(cmd.iter().collect()),
                                )?;
                            }
                        }
                        KeyCode::Down => {
                            if history_idx < history.len() {
                                history_idx += 1;
                                if history_idx == history.len() {
                                    cmd = mem_cmd.clone()
                                } else {
                                    cmd = history[history_idx].clone()
                                }
                                line_pos = cmd.len();
                                queue!(
                                    output,
                                    cursor::MoveTo(base_pos.0, base_pos.1),
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                    Print::<String>(cmd.iter().collect()),
                                )?;
                            }
                        }
                        KeyCode::Enter => {
                            if 0 < cmd.len() {
                                // Append command to history and reset status
                                history.push(cmd);
                                cmd = vec![];
                                line_pos = 0;
                                history_idx = history.len();
                                queue!(
                                    output,
                                    cursor::MoveTo(base_pos.0, base_pos.1),
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                )?;
                                // Parse the command
                                let cmd_str: String = history[history.len() - 1].iter().collect();
                                self.parse_cmd(&cmd_str[..]);
                            }
                        }
                        KeyCode::Esc => break,
                        _ => (),
                    };
                    self.flush();
                }
                Event::Resize(width, height) => self.resize((width, height)),
                _ => (),
            }
        }

        Ok(())
    }

    fn parse_cmd(&mut self, cmd: &str) -> () {
        for command in &self.commands {
            match command.match_cmd(cmd) {
                Some(mapping) => {
                    match command.get_keyword() {
                        RUN => {
                            let nb_gens = *mapping.get("nb_gens").unwrap();
                            match nb_gens.parse::<u64>() {
                                Ok(nb_gens) => self.run(nb_gens),
                                Err(_) => (), // Print error on terminal here
                            }
                        }
                        GOTO => {
                            let cur_gen = self.simulator.current_gen();
                            let target_gen = *mapping.get("target_gen").unwrap();
                            match target_gen.parse::<u64>() {
                                Ok(target_gen) if target_gen > cur_gen => {
                                    self.run(target_gen - cur_gen)
                                }
                                Ok(_) => (),  // Print error on terminal here
                                Err(_) => (), // Print error on terminal here
                            }
                        }
                        VIEW => {
                            let x_arg = *mapping.get("x").unwrap();
                            let y_arg = *mapping.get("y").unwrap();
                            if let Ok(x) = x_arg.parse::<usize>() {
                                if let Ok(y) = y_arg.parse::<usize>() {
                                    self.move_view(x, y);
                                } else {
                                    // Print error on terminal here
                                }
                            } else {
                                // Print error on terminal here
                            }
                        }
                        _ => panic!("Unsupported command."),
                    }
                    break;
                }
                None => (),
            }
        }
    }

    fn run(&mut self, nb_gens: u64) -> () {
        // Update title
        let mut new_title = self.auto_mod.get_title().clone();
        new_title.push(
            style(format!(
                " (running to generation {})",
                (self.simulator.current_gen() + nb_gens).to_string()
            ))
            .attribute(Attribute::SlowBlink)
            .attribute(Attribute::Italic),
        );
        self.auto_mod.set_title(new_title);

        // Run the simulator
        for _i in 0..nb_gens {
            self.simulator.run(1);
            self.draw_automaton();
            thread::sleep(time::Duration::from_millis(100));
        }

        // Reset title to original
        let mut title = self.auto_mod.get_title().clone();
        title.pop();
        self.auto_mod.set_title(title);

        self.cursor_to_command();
        self.flush();
    }

    fn move_view(&mut self, x: usize, y: usize) -> () {
        let dim = self.simulator.size();
        if x < dim.nb_cols && y < dim.nb_rows {
            self.view.0 = x;
            self.view.1 = y;
            self.draw_automaton();
        }
    }

    fn draw_automaton(&self) -> () {
        // Get maximum render size and convert to (usize, usize)
        let max_render_size = self.auto_mod.get_render_size();
        let max_render_size = (max_render_size.0 as usize, max_render_size.1 as usize);

        // Determine real render size
        let dim = self.simulator.size();
        let mut render_size = (dim.nb_cols - self.view.0, dim.nb_rows - self.view.1);
        if render_size.0 > max_render_size.0 {
            render_size.0 = max_render_size.0;
        }
        if render_size.1 > max_render_size.1 {
            render_size.1 = max_render_size.1;
        }

        // Clear module content and redraw over it
        self.auto_mod.clear_content();

        let render_pos = self.auto_mod.get_render_pos();
        let mut row = self.view.1;
        let mut stdout = stdout();
        for y in 0..render_size.1 {
            queue!(
                stdout,
                cursor::MoveTo(render_pos.0, render_pos.1 + (y as u16))
            )
            .expect("Failed to move cursor.");
            for x in 0..render_size.0 {
                let c = self.simulator.get_cell(&Position::new(self.view.0 + x, row)).style().clone();
                queue!(stdout, PrintStyledContent(c)).expect("Failed to display simulator");
            }
            // Next row
            row += 1
        }

        // Update info module
        let auto_size = self.simulator.size();
        let (x, y) = self.info_mod.get_render_pos();
        let (max_len, _) = self.info_mod.get_render_size();

        let generation = StyledText::from(vec![
            style(String::from(" Generation: ")).attribute(Attribute::Italic),
            style(self.simulator.current_gen().to_string()),
        ]);
        let size = StyledText::from(vec![
            style(String::from(" Total size: ")).attribute(Attribute::Italic),
            style(format!(
                "({}, {})",
                auto_size.nb_rows.to_string(),
                auto_size.nb_cols.to_string()
            )),
        ]);
        let view = StyledText::from(vec![
            style(String::from(" Viewing   : ")).attribute(Attribute::Italic),
            style(format!(
                "({}, {}) -> ({}, {})",
                self.view.0.to_string(),
                self.view.1.to_string(),
                (self.view.0 + render_size.0).to_string(),
                (self.view.1 + render_size.1).to_string(),
            )),
        ]);

        // Draw
        self.info_mod.clear_content();
        generation.draw(&mut stdout, cursor::MoveTo(x, y + 1), max_len);
        size.draw(&mut stdout, cursor::MoveTo(x, y + 3), max_len);
        view.draw(&mut stdout, cursor::MoveTo(x, y + 5), max_len);
        self.cursor_to_command();
        self.flush();
    }

    fn resize(&mut self, size: Size) -> () {
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");

        // Recreate modules
        let mut new_modules = Self::create_modules(size);
        new_modules.0.set_title(self.auto_mod.get_title().clone());
        new_modules.1.set_title(self.info_mod.get_title().clone());
        self.auto_mod = new_modules.0;
        self.info_mod = new_modules.1;

        // Return cursor to command
        self.draw_automaton();
        self.cursor_to_command();
        self.flush();
    }

    fn create_modules(size: Size) -> (Module, Module) {
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

    fn cursor_to_command(&self) -> () {
        queue!(stdout(), cursor::MoveTo(0, self.size.1 - 1), Print("> "))
            .expect("Failed to move cursor to command line.");
    }

    fn flush(&self) -> () {
        stdout().flush().expect("Failed to flush stdout.");
    }
}
