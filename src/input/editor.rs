pub struct Editor {
    buffer: Vec<char>,
    cursor: usize,
}

impl Editor {
    // Create empty editor state
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
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

    // Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn replace_range(
        &mut self,
        start: usize,
        end: usize,
        replacement: &str,
    ) {
        self.buffer.splice(
            start..end,
            replacement.chars(),
        );

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

        self.buffer.splice(
            start..self.cursor,
            replacement.chars(),
        );

        self.cursor = start + replacement.chars().count();
    }

}
