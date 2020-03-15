use crossterm::{
    cursor, queue,
    style::{PrintStyledContent, StyledContent},
};
use std::io::{Stdout, Write};

#[derive(Clone)]
pub struct StyledText {
    text: Vec<StyledContent<String>>,
}

impl StyledText {
    pub fn new() -> Self {
        Self { text: vec![] }
    }

    pub fn from(text: Vec<StyledContent<String>>) -> Self {
        Self { text }
    }

    pub fn pop(&mut self) -> () {
        self.text.pop();
    }

    pub fn push(&mut self, content: StyledContent<String>) -> () {
        self.text.push(content);
    }

    pub fn update(&mut self, index: usize, content: StyledContent<String>) -> () {
        self.text[index] = content;
    }

    pub fn draw(&self, stdout: &mut Stdout, pos: cursor::MoveTo, max_len: u16) -> u16 {
        // Move cursor to correct position
        queue!(stdout, pos).expect("Failed to move cursor.");
        let mut total_len = 0;
        for elem in &self.text {
            let elem_len = elem.content().chars().count();

            // Print content or stop if the line is full
            if total_len + elem_len <= (max_len as usize) {
                queue!(stdout, PrintStyledContent(elem.clone())).expect("Failed to print content.");
                total_len += elem_len
            } else {
                break;
            }
        }

        // Return nuber of characters written
        total_len as u16
    }
}
