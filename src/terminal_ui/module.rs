use super::Position;
use crossterm::{cursor::MoveTo, queue, style::Print};
use std::io::{Stdout, Write};

use crate::terminal_ui::styled_text::StyledText;

pub struct Module {
    pub title: StyledText,
    pos: Position,
    size: Position,
}

impl Module {
    pub fn new(title: StyledText, pos: Position, size: Position) -> Self {
        if size.0 < 3 || size.1 < 3 {
            panic!("Module size must be at least 3x3.")
        }
        Module { title, pos, size }
    }

    pub fn clear_content(&self, stdout: &mut Stdout) -> () {
        let content_pos = self.get_render_pos();
        let content_size = self.get_render_size();

        let empty_line = std::iter::repeat(' ')
            .take(content_size.0 as usize)
            .collect::<String>();

        for x in 0..content_size.1 {
            queue!(
                stdout,
                MoveTo(content_pos.0, content_pos.1 + x),
                Print(empty_line.clone())
            )
            .expect("Failed to clear module content.")
        }
    }

    pub fn draw(&self, stdout: &mut Stdout) -> () {
        let err_msg = "Failed to draw module.";

        // Draw top line
        queue!(
            stdout,
            MoveTo(self.pos.0, self.pos.1),
            Print("┌─"),
            MoveTo(self.pos.0 + self.size.0 - 2, self.pos.1),
            Print("─┐"),
        )
        .expect(err_msg);

        // Draw vertical lines
        for row in (self.pos.1 + 1)..(self.pos.1 + self.size.1 - 1) {
            queue!(
                stdout,
                MoveTo(self.pos.0, row),
                Print('│'),
                MoveTo(self.pos.0 + self.size.0 - 1, row),
                Print('│')
            )
            .expect(err_msg);
        }

        // Draw bottom line
        let hline = std::iter::repeat('─')
            .take(self.size.0 as usize - 2)
            .collect::<String>();
        queue!(
            stdout,
            MoveTo(self.pos.0, self.pos.1 + self.size.1 - 1),
            Print('└'),
            Print(hline),
            Print('┘')
        )
        .expect(err_msg);

        // Draw title
        self.draw_title(stdout);
    }

    pub fn draw_title(&self, stdout: &mut Stdout) -> () {
        let err_msg = "Failed to draw module's title.";
        let max_len = self.size.0 - 4;
        let base_pos = self.pos.0 + 3;
        queue!(stdout, MoveTo(base_pos - 1, self.pos.1), Print(' '),).expect(err_msg);
        let nb_written = self
            .title
            .draw(stdout, MoveTo(base_pos, self.pos.1), max_len);
        if nb_written < max_len {
            let hline = std::iter::repeat('─')
                .take((max_len - nb_written - 1) as usize)
                .collect::<String>();
            let mut top_line = String::from(" ");
            top_line.push_str(&hline[..]);
            queue!(
                stdout,
                MoveTo(base_pos + nb_written, self.pos.1),
                Print(top_line),
            )
            .expect(err_msg);
        }
    }

    pub fn get_render_pos(&self) -> Position {
        (self.pos.0 + 1, self.pos.1 + 1)
    }

    pub fn get_render_size(&self) -> Position {
        (self.size.0 - 2, self.size.1 - 2)
    }
}
