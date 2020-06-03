// Standard library
use std::io::{stdout, Write};
use std::{thread, time};

// External libraries
use crossterm::{
    cursor,
    event::{Event, KeyCode},
    queue,
    style::{style, Attribute, Print, PrintStyledContent},
    terminal,
};

// CELL
mod module;
mod styled_text;
use crate::automaton::TermDrawableAutomaton;
use crate::commands::Command;
use crate::grid::{Dimensions, Position};
use crate::simulator::Simulator;
use module::Module;
use styled_text::StyledText;

pub struct TerminalUI<A: TermDrawableAutomaton> {
    size: Size,
    auto_mod: Module,
    info_mod: Module,
    simulator: Simulator<A>,
    current_gen: usize,
    current_grid_size: Dimensions,
    view: (u32, u32),
    commands: Vec<Command>,
}

impl<A: TermDrawableAutomaton> TerminalUI<A> {
    pub fn new(mut simulator: Simulator<A>) -> Self {
        // Clear terminal
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");

        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = Self::create_modules(size);
        let current_grid_size = *simulator.get_gen(0, false).unwrap().dim();
        let mut ui = Self {
            size,
            auto_mod: modules.0,
            info_mod: modules.1,
            simulator,
            current_gen: 0,
            current_grid_size,
            view: (0, 0),
            commands: vec![
                Command::new(RUN, vec!["nb_gens"]),
                Command::new(GOTO, vec!["target_gen"]),
                Command::new(VIEW, vec!["x", "y"]),
                Command::new(SHOW, vec!["gen"]),
            ],
        };

        // Set simulator title and draw initial state
        let title = StyledText::from(vec![style(String::from(ui.simulator.automaton().name()))]);
        ui.auto_mod.set_title(title);
        ui.draw_automaton(0);
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
                            match nb_gens.parse::<usize>() {
                                Ok(nb_gens) => self.goto(self.current_gen + nb_gens),
                                Err(_) => (), // Print error on terminal here
                            }
                        }
                        GOTO => {
                            let target_gen = *mapping.get("target_gen").unwrap();
                            match target_gen.parse::<usize>() {
                                Ok(target_gen) => self.goto(target_gen),
                                Err(_) => (), // Print error on terminal here
                            }
                        }
                        VIEW => {
                            let x_arg = *mapping.get("x").unwrap();
                            let y_arg = *mapping.get("y").unwrap();
                            if let Ok(x) = x_arg.parse::<u32>() {
                                if let Ok(y) = y_arg.parse::<u32>() {
                                    self.move_view(x, y);
                                } else {
                                    // Print error on terminal here
                                }
                            } else {
                                // Print error on terminal here
                            }
                        }
                        SHOW => {
                            let gen = *mapping.get("gen").unwrap();
                            match gen.parse::<usize>() {
                                Ok(gen) => self.draw_automaton(gen),
                                Err(_) => (), // Print error on terminal here
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

    fn goto(&mut self, target_gen: usize) -> () {
        // Update title
        let mut new_title = self.auto_mod.get_title().clone();
        new_title.push(
            style(format!(
                " (running to generation {})",
                target_gen.to_string()
            ))
            .attribute(Attribute::SlowBlink)
            .attribute(Attribute::Italic),
        );
        self.auto_mod.set_title(new_title);

        if target_gen <= self.current_gen {
            self.draw_automaton(target_gen);
        } else {
            // Launch asynchronous computations and draw each new generation
            self.simulator.goto(target_gen);
            for i in self.current_gen..target_gen {
                self.draw_automaton(i + 1);
                thread::sleep(time::Duration::from_millis(100));
            }
        }

        // Reset title to original
        let mut title = self.auto_mod.get_title().clone();
        title.pop();
        self.auto_mod.set_title(title);

        self.cursor_to_command();
        self.flush();
    }

    fn move_view(&mut self, x: u32, y: u32) -> () {
        if x < self.current_grid_size.width() && y < self.current_grid_size.height() {
            self.view.0 = x;
            self.view.1 = y;
            self.draw_automaton(self.current_gen);
        }
    }

    fn draw_automaton(&mut self, gen: usize) -> () {
        // Get generation's grid and update state
        let grid = self.simulator.get_gen(gen, true).unwrap();
        self.current_grid_size = *grid.dim();
        self.current_gen = gen;

        // Get maximum render size and convert to (usize, usize)
        let max_render_size = self.auto_mod.get_render_size();
        let max_render_size = (max_render_size.0 as u32, max_render_size.1 as u32);

        // Determine real render size
        let mut render_size = (
            self.current_grid_size.width() - self.view.0,
            self.current_grid_size.height() - self.view.1,
        );
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
                let state = grid.get(Position::new(self.view.0 + x, row));
                let c = state.style();
                queue!(stdout, PrintStyledContent(c)).expect("Failed to display simulator");
            }
            // Next row
            row += 1
        }

        // Update info module
        let (x, y) = self.info_mod.get_render_pos();
        let (max_len, _) = self.info_mod.get_render_size();

        let generation = StyledText::from(vec![
            style(String::from(" Generation: ")).attribute(Attribute::Italic),
            style(gen.to_string()),
        ]);
        let size = StyledText::from(vec![
            style(String::from(" Total size: ")).attribute(Attribute::Italic),
            style(format!(
                "{} x {}",
                self.current_grid_size.width().to_string(),
                self.current_grid_size.height().to_string()
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
        self.draw_automaton(self.current_gen);
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

type Size = (u16, u16);

const HEIGHT_INFO: u16 = 10;
const RUN: &str = "run";
const GOTO: &str = "goto";
const VIEW: &str = "view";
const SHOW: &str = "show";
