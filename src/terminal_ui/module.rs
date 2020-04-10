use super::Size;
use crossterm::{cursor::MoveTo, queue, style::Print};
use std::io::{stdout, Write};

use crate::terminal_ui::styled_text::StyledText;

pub struct Module {
    title: StyledText,
    pos: Size,
    size: Size,
}

impl Module {
    pub fn new(title: StyledText, pos: Size, size: Size) -> Self {
        if size.0 < 3 || size.1 < 3 {
            panic!("Module size must be at least 3x3.")
        }
        let module = Self { title, pos, size };
        module.draw();
        module
    }

    pub fn set_title(&mut self, title: StyledText) {
        self.title = title;
        self.draw_title();
    }

    pub fn clear(&mut self) -> () {
        let empty_line = std::iter::repeat(' ')
            .take(self.size.0 as usize)
            .collect::<String>();

        for x in 0..self.size.1 {
            queue!(
                stdout(),
                MoveTo(self.pos.0, self.pos.1 + x),
                Print(empty_line.clone())
            )
            .expect("Failed to clear module content.")
        }
    }

    pub fn clear_content(&self) -> () {
        let content_pos = self.get_render_pos();
        let content_size = self.get_render_size();

        let empty_line = std::iter::repeat(' ')
            .take(content_size.0 as usize)
            .collect::<String>();

        for x in 0..content_size.1 {
            queue!(
                stdout(),
                MoveTo(content_pos.0, content_pos.1 + x),
                Print(empty_line.clone())
            )
            .expect("Failed to clear module content.")
        }
    }

    pub fn draw(&self) -> () {
        self.draw_box();
        self.draw_title();
    }

    pub fn draw_box(&self) -> () {
        let err_msg = "Failed to draw module.";
        let mut output = stdout();

        // Draw top line
        queue!(
            output,
            MoveTo(self.pos.0, self.pos.1),
            Print("┌─"),
            MoveTo(self.pos.0 + self.size.0 - 2, self.pos.1),
            Print("─┐"),
        )
        .expect(err_msg);

        // Draw vertical lines
        for row in (self.pos.1 + 1)..(self.pos.1 + self.size.1 - 1) {
            queue!(
                output,
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
            output,
            MoveTo(self.pos.0, self.pos.1 + self.size.1 - 1),
            Print('└'),
            Print(hline),
            Print('┘')
        )
        .expect(err_msg);
    }

    pub fn draw_title(&self) -> () {
        let mut output = stdout();
        let err_msg = "Failed to draw module's title.";
        let max_len = self.size.0 - 4;
        let base_pos = self.pos.0 + 3;
        queue!(output, MoveTo(base_pos - 1, self.pos.1), Print(' '),).expect(err_msg);
        let nb_written = self
            .title
            .draw(&mut output, MoveTo(base_pos, self.pos.1), max_len);
        if nb_written < max_len {
            let hline = std::iter::repeat('─')
                .take((max_len - nb_written - 1) as usize)
                .collect::<String>();
            let mut top_line = String::from(" ");
            top_line.push_str(&hline[..]);
            queue!(
                output,
                MoveTo(base_pos + nb_written, self.pos.1),
                Print(top_line),
            )
            .expect(err_msg);
        }
    }

    pub fn get_title(&self) -> &StyledText {
        &self.title
    }

    pub fn get_render_pos(&self) -> Size {
        (self.pos.0 + 1, self.pos.1 + 1)
    }

    pub fn get_render_size(&self) -> Size {
        (self.size.0 - 2, self.size.1 - 2)
    }
}
