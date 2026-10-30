use std::cell::Cell;

pub struct Editor {
    buffer: Vec<char>,
    cursor: usize,
    input_start_column: usize,
    rendered_cursor_row: Cell<usize>,
}

impl Editor {
    // Create empty editor state
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
            input_start_column: 0,
            rendered_cursor_row: Cell::new(0),
        }
    }

    // Insert character at cursor position
    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);

        self.cursor += 1;
    }

    // Remove character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;

            self.buffer.remove(self.cursor);
        }
    }

    // Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    // Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    // Move cursor to the beginning
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    // Move cursor to the end
    pub fn move_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    pub fn delete_word_backward(&mut self) {
        let mut start = self.cursor;

        while start > 0 && self.buffer[start - 1].is_whitespace() {
            start -= 1;
        }

        while start > 0 && !self.buffer[start - 1].is_whitespace() {
            start -= 1;
        }

        self.buffer.drain(start..self.cursor);
        self.cursor = start;
    }

    pub fn delete_to_end(&mut self) {
        self.buffer.truncate(self.cursor);
    }

    // Replace current text
    pub fn set_text(&mut self, text: &str) {
        self.buffer = text.chars().collect();

        self.cursor = self.buffer.len();
    }

    // Get current text
    pub fn text(&self) -> String {
        self.buffer.iter().collect()
    }

    // Get cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_input_start_column(&mut self, column: usize) {
        self.input_start_column = column;
    }

    pub fn input_start_column(&self) -> usize {
        self.input_start_column
    }

    pub fn rendered_cursor_row(&self) -> usize {
        self.rendered_cursor_row.get()
    }

    pub fn set_rendered_cursor_row(&self, row: usize) {
        self.rendered_cursor_row.set(row);
    }

    pub fn set_cursor(&mut self, position: usize) {
        self.cursor = position.min(self.buffer.len());
    }

    // Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn replace_range(&mut self, start: usize, end: usize, replacement: &str) {
        self.buffer.splice(start..end, replacement.chars());

        self.cursor = start + replacement.chars().count();
    }

    pub fn current_word_start(&self) -> usize {
        let mut index = self.cursor;

        while index > 0 {
            if self.buffer[index - 1].is_whitespace() {
                break;
            }

            index -= 1;
        }

        index
    }

    pub fn replace_current_word(&mut self, replacement: &str) {
        let start = self.current_word_start();

        self.buffer.splice(start..self.cursor, replacement.chars());

        self.cursor = start + replacement.chars().count();
    }
}
