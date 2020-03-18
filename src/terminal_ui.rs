use crate::automaton::{Cells, CellularAutomaton, Operation as AutoOP};
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

mod module;
mod styled_text;

use module::Module;
use styled_text::StyledText;

type Size = (u16, u16);

const HEIGHT_INFO: u16 = 10;

pub struct TerminalUI<C: Cells + PartialEq + Eq + Hash> {
    stdout: Stdout,
    size: Size,
    auto_mod: Module,
    info_mod: Module,
    auto_offset: (usize, usize),
    state: State,
    info: Option<AutomatonInfo<C>>,
}

impl<C: Cells + PartialEq + Eq + Hash> TerminalUI<C> {
    pub fn new() -> Self {
        // Clear terminal
        let mut stdout = stdout();
        queue!(stdout, terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");

        let size = terminal::size().expect("Failed to read terminal size.");
        let modules = Self::create_modules(size);
        let mut ui = Self {
            stdout,
            size,
            auto_mod: modules.0,
            info_mod: modules.1,
            auto_offset: (0, 0),
            state: State::NoAutomaton,
            info: None,
        };

        ui.cursor_to_command();
        ui.flush();
        ui
    }

    pub fn cmd_interpreter(
        &mut self,
        automaton: &mut CellularAutomaton<C>,
    ) -> crossterm::Result<()> {
        // Ensure cursor is on command line
        self.cursor_to_command();
        let base_pos = cursor::position()?;
        let max_len = self.size.0 - base_pos.0;

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
                                    self.stdout,
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
                                queue!(self.stdout, cursor::MoveLeft(1))?;
                                line_pos -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if line_pos < cmd.len() {
                                queue!(self.stdout, cursor::MoveRight(1))?;
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
                                    self.stdout,
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
                                    self.stdout,
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
                                    self.stdout,
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
                                    self.stdout,
                                    cursor::MoveTo(base_pos.0, base_pos.1),
                                    terminal::Clear(terminal::ClearType::UntilNewLine),
                                    Print::<String>(cmd.iter().collect()),
                                )?;
                            }
                        }
                        KeyCode::Enter => {
                            if 0 < cmd.len() {
                                // Append command to history
                                // let cmd_str: String = cmd.iter().collect();
                                history.push(cmd);

                                // Reset status
                                cmd = vec![];
                                line_pos = 0;
                                history_idx = history.len();

                                // Reset command line
                                queue!(
                                    self.stdout,
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
                Event::Resize(width, height) => self.perform(Operation::Resize(width, height)),
                _ => (),
            }
        }

        Ok(())
    }

    pub fn perform<'a>(&mut self, op: Operation<'a, C>) -> () {
        match self.state {
            State::NoAutomaton => match op {
                Operation::BindAutomaton(automaton, printer) => {
                    self.bind_automaton(automaton, printer)
                }
                Operation::Resize(width, height) => self.resize((width, height)),
                _ => panic!("Unsupported operation."),
            },
            State::Binded => match op {
                Operation::BindAutomaton(automaton, printer) => {
                    self.bind_automaton(automaton, printer)
                }
                Operation::SetState(automaton) => self.update_gen(automaton),
                Operation::NotifyEvolution(target_gen) => self.notify_evolution(target_gen),
                Operation::Unbind => self.unbind(),
                Operation::Resize(width, height) => self.resize((width, height)),
            },
            State::Running(target_gen) => match op {
                Operation::SetState(automaton) => {
                    if automaton.current_gen() >= target_gen {
                        // Update state and title
                        self.state = State::Binded;

                        let mut title = self.auto_mod.get_title().clone();
                        title.pop();
                        self.auto_mod.set_title(title);
                    }
                    self.update_gen(automaton)
                }
                Operation::Resize(width, height) => self.resize((width, height)),
                _ => panic!("Unsupported operation."),
            },
        }

        self.cursor_to_command();
        self.flush();
    }

    fn bind_automaton(
        &mut self,
        automaton: &CellularAutomaton<C>,
        printer: HashMap<C, StyledContent<char>>,
    ) -> () {
        // Update state
        self.state = State::Binded;

        // Change automaton title and update automaton info
        let info = AutomatonInfo::new(automaton, printer);
        self.auto_mod
            .set_title(StyledText::from(vec![style(info.name.clone())]));
        self.info = Some(info);

        // Draw automaton
        self.draw_automaton(automaton);
    }

    fn update_gen(&mut self, automaton: &CellularAutomaton<C>) -> () {
        let auto_gen = automaton.current_gen();
        let mut info = self.info.as_mut().unwrap();
        if auto_gen < info.current_gen {
            panic!("Trying to rollback automaton.")
        }
        info.current_gen = auto_gen;
        self.draw_automaton(automaton)
    }

    fn notify_evolution(&mut self, target_gen: u64) -> () {
        if target_gen <= self.info.as_mut().unwrap().current_gen {
            panic!("Evolution cannot rollback automaton.")
        }

        // Update state and change automaton title
        self.state = State::Running(target_gen);

        let mut new_title = self.auto_mod.get_title().clone();
        new_title.push(
            style(format!(" - running (to gen. {})", target_gen.to_string()))
                .attribute(Attribute::SlowBlink)
                .attribute(Attribute::Italic),
        );
        self.auto_mod.set_title(new_title);
    }

    fn unbind(&mut self) -> () {
        // Update state and automaton module's title
        self.state = State::NoAutomaton;
        self.auto_mod
            .set_title(StyledText::from(vec![style(String::from("Automaton"))]));
    }

    fn draw_automaton(&mut self, automaton: &CellularAutomaton<C>) -> () {
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
        let info = self.info.as_ref().unwrap();
        let (x, y) = self.info_mod.get_render_pos();
        let (max_len, _) = self.info_mod.get_render_size();

        let generation = StyledText::from(vec![
            style(String::from(" Generation: ")).attribute(Attribute::Italic),
            style(info.current_gen.to_string()),
        ]);
        let size = StyledText::from(vec![
            style(String::from(" Total size: ")).attribute(Attribute::Italic),
            style(format!(
                "({}, {})",
                info.size.0.to_string(),
                info.size.1.to_string()
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
        generation.draw(&mut self.stdout, cursor::MoveTo(x, y + 1), max_len);
        size.draw(&mut self.stdout, cursor::MoveTo(x, y + 3), max_len);
        view.draw(&mut self.stdout, cursor::MoveTo(x, y + 5), max_len);
    }

    fn resize(&mut self, size: Size) -> () {
        queue!(self.stdout, terminal::Clear(terminal::ClearType::All))
            .expect("Failed to clear terminal.");

        // Recreate modules
        let mut new_modules = Self::create_modules(size);
        new_modules.0.set_title(self.auto_mod.get_title().clone());
        new_modules.1.set_title(self.info_mod.get_title().clone());
        self.auto_mod = new_modules.0;
        self.info_mod = new_modules.1;

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

    fn cursor_to_command(&mut self) -> () {
        queue!(self.stdout, cursor::MoveTo(0, self.size.1 - 1), Print("> "))
            .expect("Failed to move cursor to command line.");
    }

    fn flush(&mut self) -> () {
        self.stdout.flush().expect("Failed to flush stdout.");
    }
}

pub enum Operation<'a, C: Cells> {
    BindAutomaton(&'a CellularAutomaton<C>, HashMap<C, StyledContent<char>>),
    SetState(&'a CellularAutomaton<C>),
    NotifyEvolution(u64),
    Unbind,
    Resize(u16, u16),
}

enum State {
    NoAutomaton,
    Binded,
    Running(u64),
}

struct AutomatonInfo<C: Cells + PartialEq + Eq + Hash> {
    name: String,
    size: (usize, usize),
    current_gen: u64,
    printer: HashMap<C, StyledContent<char>>,
}

impl<C: Cells + PartialEq + Eq + Hash> AutomatonInfo<C> {
    fn new(
        automaton: &CellularAutomaton<C>,
        printer: HashMap<C, StyledContent<char>>,
    ) -> AutomatonInfo<C> {
        AutomatonInfo {
            name: String::from("dummy"),
            size: automaton.size(),
            current_gen: automaton.current_gen(),
            printer,
        }
    }
}
