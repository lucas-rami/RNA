use crate::automaton::{Cells, CellularAutomaton, Operation};
use crate::commands::Command;
use crossterm::{
    cursor,
    event::{Event, KeyCode},
    queue,
    style::{style, Attribute, Print, PrintStyledContent, StyledContent},
    terminal,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{stdout, Stdout, Write};
use std::{thread, time};

mod module;
mod styled_text;

use module::Module;
use styled_text::StyledText;

type Size = (u16, u16);

const HEIGHT_INFO: u16 = 10;
const RUN: &str = "run";
const GOTO: &str = "goto";

pub struct TerminalUI<C: Cells + PartialEq + Eq + Hash> {
    size: Size,
    auto_mod: Module,
    info_mod: Module,
    auto_offset: (usize, usize),
    info: Option<AutomatonInfo<C>>,
    commands: Vec<Command>,
}

impl<C: Cells + PartialEq + Eq + Hash> TerminalUI<C> {
    pub fn new() -> Self {
        // Clear terminal
        queue!(stdout(), terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");

        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = Self::create_modules(size);
        let ui = Self {
            size,
            auto_mod: modules.0,
            info_mod: modules.1,
            auto_offset: (0, 0),
            info: None,
            commands: vec![
                Command::new(RUN, vec!["nb_gens"]),
                Command::new(GOTO, vec!["target_gen"]),
            ],
        };

        ui.cursor_to_command();
        ui.flush();
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
                                // Parse the command
                                let cmd_str: String = cmd.iter().collect();
                                self.parse_cmd(&cmd_str[..]);

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
                            if let Some(info) = &self.info {
                                let cur_gen = info.automaton.current_gen();
                                let target_gen = *mapping.get("target_gen").unwrap();
                                match target_gen.parse::<u64>() {
                                    Ok(target_gen) if target_gen > cur_gen => {
                                        self.run(target_gen - cur_gen)
                                    }
                                    Ok(_) => (),  // Print error
                                    Err(_) => (), // Print error on terminal here
                                }
                            } else {
                                // Print error
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

    pub fn bind_automaton(
        &mut self,
        automaton: CellularAutomaton<C>,
        printer: HashMap<C, StyledContent<char>>,
    ) -> () {
        if !automaton.is_ready() {
            panic!("Automaton isn't initialized.")
        }

        // Change automaton title on UI and update automaton info
        let name = style(String::from(automaton.get_name()));
        self.auto_mod.set_title(StyledText::from(vec![name]));
        self.info = Some(AutomatonInfo::new(automaton, printer));

        // Draw automaton
        self.draw_automaton();
    }

    fn run(&mut self, nb_gens: u64) -> () {
        let automaton = &self.info.as_mut().unwrap().automaton;

        // Update title
        let mut new_title = self.auto_mod.get_title().clone();
        new_title.push(
            style(format!(
                " (running to generation {})",
                (automaton.current_gen() + nb_gens).to_string()
            ))
            .attribute(Attribute::SlowBlink)
            .attribute(Attribute::Italic),
        );
        self.auto_mod.set_title(new_title);

        // Run the automaton
        for _i in 0..nb_gens {
            let automaton = &mut self.info.as_mut().unwrap().automaton;
            automaton.perform(Operation::Step);
            self.draw_automaton();
            thread::sleep(time::Duration::from_millis(300));
        }

        // Reset title to original
        let mut title = self.auto_mod.get_title().clone();
        title.pop();
        self.auto_mod.set_title(title);

        self.cursor_to_command();
        self.flush();
    }

    fn draw_automaton(&self) -> () {
        let automaton = &self.info.as_ref().unwrap().automaton;
        // Get maximum render size and convert to (usize, usize)
        let max_render_size = self.auto_mod.get_render_size();
        let max_render_size = (max_render_size.0 as usize, max_render_size.1 as usize);

        // Determine real render size
        let auto_size = automaton.size();
        let mut render_size = (
            auto_size.0 - self.auto_offset.0,
            auto_size.1 - self.auto_offset.1,
        );
        if render_size.0 > max_render_size.0 {
            render_size.0 = max_render_size.0;
        }
        if render_size.1 > max_render_size.1 {
            render_size.1 = max_render_size.1;
        }

        // Clear module content and redraw over it
        self.auto_mod.clear_content();

        let printer = &self.info.as_ref().unwrap().printer;
        let render_pos = self.auto_mod.get_render_pos();
        let mut row = self.auto_offset.1;
        let mut stdout = stdout();
        for y in 0..render_size.1 {
            queue!(
                stdout,
                cursor::MoveTo(render_pos.0, render_pos.1 + (y as u16))
            )
            .expect("Failed to move cursor.");
            for x in 0..render_size.0 {
                let c = match printer.get(automaton.get_cell(row, self.auto_offset.0 + x)) {
                    Some(repr) => repr.clone(),
                    None => style('?'),
                };
                queue!(stdout, PrintStyledContent(c)).expect("Failed to display automaton");
            }
            // Next row
            row += 1
        }

        // Update info module
        let auto_size = automaton.size();
        let (x, y) = self.info_mod.get_render_pos();
        let (max_len, _) = self.info_mod.get_render_size();

        let generation = StyledText::from(vec![
            style(String::from(" Generation: ")).attribute(Attribute::Italic),
            style(automaton.current_gen().to_string()),
        ]);
        let size = StyledText::from(vec![
            style(String::from(" Total size: ")).attribute(Attribute::Italic),
            style(format!(
                "({}, {})",
                auto_size.0.to_string(),
                auto_size.1.to_string()
            )),
        ]);
        let view = StyledText::from(vec![
            style(String::from(" Viewing   : ")).attribute(Attribute::Italic),
            style(format!(
                "({}, {}) -> ({}, {})",
                self.auto_offset.0.to_string(),
                self.auto_offset.1.to_string(),
                (self.auto_offset.0 + render_size.0).to_string(),
                (self.auto_offset.1 + render_size.1).to_string(),
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

        // Redraw automaton if binded
        if let Some(_) = &self.info {
            self.draw_automaton();
        }

        // Return cursor to command
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

struct AutomatonInfo<C: Cells + PartialEq + Eq + Hash> {
    automaton: CellularAutomaton<C>,
    printer: HashMap<C, StyledContent<char>>,
}

impl<C: Cells + PartialEq + Eq + Hash> AutomatonInfo<C> {
    fn new(
        automaton: CellularAutomaton<C>,
        printer: HashMap<C, StyledContent<char>>,
    ) -> AutomatonInfo<C> {
        AutomatonInfo { automaton, printer }
    }
}
