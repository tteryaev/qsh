use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::config::Theme;

use super::{completion, editor::Editor, highlight, render, width};

use std::io::{self, Write};

struct CompletionState {
    matches: Vec<String>,
    selected_index: Option<usize>,
    word_start: usize,
    word_end: usize,
    word: String,
}

impl CompletionState {
    fn new(matches: Vec<String>, word_start: usize, word_end: usize, word: String) -> Self {
        Self {
            matches,
            selected_index: None,
            word_start,
            word_end,
            word,
        }
    }

    fn is_active_for(&self, editor: &Editor) -> bool {
        self.word_start == editor.current_word_start()
            && self.word_end == editor.cursor()
            && self.word == editor_word(editor, self.word_start, self.word_end)
    }

    fn select_next(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = None;
            return;
        }

        self.selected_index = Some(match self.selected_index {
            Some(index) => (index + 1) % self.matches.len(),
            None => 0,
        });
    }

    fn select_previous(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = None;
            return;
        }

        self.selected_index = Some(match self.selected_index {
            Some(0) | None => self.matches.len() - 1,
            Some(index) => index - 1,
        });
    }

    fn selected(&self) -> Option<&str> {
        self.selected_index
            .and_then(|index| self.matches.get(index))
            .map(String::as_str)
    }
}

fn clear_completions(completion_state: &mut Option<CompletionState>, completion_lines: &mut usize) {
    clear_completion_lines(completion_lines);

    *completion_state = None;
}

pub fn read_input(prompt: &str, history: &[String], theme: &Theme) -> Option<String> {
    enable_raw_mode().unwrap();

    let mut editor = Editor::new();

    let mut history_index = history.len();

    let mut browsing_history = false;

    let mut completion_state: Option<CompletionState> = None;
    let mut completion_lines = 0usize;

    loop {
        match event::read().unwrap() {
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => {
                    clear_completions(&mut completion_state, &mut completion_lines);

                    editor.insert(c);

                    browsing_history = false;

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Backspace => {
                    clear_completions(&mut completion_state, &mut completion_lines);

                    editor.backspace();

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Left => {
                    if completion_state
                        .as_ref()
                        .is_some_and(|state| state.is_active_for(&editor))
                    {
                        if let Some(state) = completion_state.as_mut() {
                            state.select_previous();
                            clear_completion_lines(&mut completion_lines);
                            completion_lines = render_completion_menu(
                                prompt,
                                &editor,
                                theme,
                                &state.matches,
                                state.selected_index,
                            );
                        }

                        continue;
                    }

                    clear_completions(&mut completion_state, &mut completion_lines);

                    editor.move_left();

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Right => {
                    if completion_state
                        .as_ref()
                        .is_some_and(|state| state.is_active_for(&editor))
                    {
                        if let Some(state) = completion_state.as_mut() {
                            state.select_next();
                            clear_completion_lines(&mut completion_lines);
                            completion_lines = render_completion_menu(
                                prompt,
                                &editor,
                                theme,
                                &state.matches,
                                state.selected_index,
                            );
                        }

                        continue;
                    }

                    clear_completions(&mut completion_state, &mut completion_lines);

                    editor.move_right();

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Home => {
                    clear_completions(&mut completion_state, &mut completion_lines);

                    editor.move_home();

                    redraw(prompt, &editor, theme);
                }

                KeyCode::End => {
                    clear_completions(&mut completion_state, &mut completion_lines);

                    editor.move_end();

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Up => {
                    if completion_state
                        .as_ref()
                        .is_some_and(|state| state.is_active_for(&editor))
                    {
                        if let Some(state) = completion_state.as_mut() {
                            state.select_previous();
                            clear_completion_lines(&mut completion_lines);
                            completion_lines = render_completion_menu(
                                prompt,
                                &editor,
                                theme,
                                &state.matches,
                                state.selected_index,
                            );
                        }

                        continue;
                    }

                    if history.is_empty() {
                        continue;
                    }

                    clear_completions(&mut completion_state, &mut completion_lines);

                    browsing_history = true;

                    if history_index > 0 {
                        history_index -= 1;
                    }

                    editor.set_text(&history[history_index]);

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Down => {
                    if completion_state
                        .as_ref()
                        .is_some_and(|state| state.is_active_for(&editor))
                    {
                        if let Some(state) = completion_state.as_mut() {
                            state.select_next();
                            clear_completion_lines(&mut completion_lines);
                            completion_lines = render_completion_menu(
                                prompt,
                                &editor,
                                theme,
                                &state.matches,
                                state.selected_index,
                            );
                        }

                        continue;
                    }

                    if !browsing_history {
                        continue;
                    }

                    clear_completions(&mut completion_state, &mut completion_lines);

                    if history_index + 1 < history.len() {
                        history_index += 1;

                        editor.set_text(&history[history_index]);
                    } else {
                        history_index = history.len();

                        editor.set_text("");
                    }

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Esc => {
                    if completion_state.is_some() {
                        clear_completions(&mut completion_state, &mut completion_lines);
                        redraw(prompt, &editor, theme);

                        continue;
                    }

                    clear_completions(&mut completion_state, &mut completion_lines);

                    browsing_history = false;
                    history_index = history.len();

                    editor.set_text("");

                    redraw(prompt, &editor, theme);
                }

                KeyCode::Tab => {
                    if completion_state
                        .as_ref()
                        .is_some_and(|state| state.is_active_for(&editor))
                    {
                        if let Some(state) = completion_state.as_mut() {
                            state.select_next();
                            clear_completion_lines(&mut completion_lines);
                            completion_lines = render_completion_menu(
                                prompt,
                                &editor,
                                theme,
                                &state.matches,
                                state.selected_index,
                            );
                        }

                        continue;
                    }

                    clear_completions(&mut completion_state, &mut completion_lines);

                    let input = editor.text();

                    let matches = completion::complete(&input, editor.cursor());

                    if matches.is_empty() {
                        continue;
                    }

                    if matches.len() == 1 {
                        editor.replace_current_word(&matches[0]);

                        redraw(prompt, &editor, theme);

                        continue;
                    }

                    let current_word_start = editor.current_word_start();

                    let current_word_length = editor.cursor() - current_word_start;

                    let prefix = completion::common_prefix(&matches);

                    let prefix_length = prefix.chars().count();

                    if prefix_length > current_word_length {
                        editor.replace_current_word(&prefix);
                    }

                    completion_state = Some(CompletionState::new(
                        matches,
                        current_word_start,
                        editor.cursor(),
                        editor_word(&editor, current_word_start, editor.cursor()),
                    ));

                    completion_lines = render_completion_menu(
                        prompt,
                        &editor,
                        theme,
                        &completion_state.as_ref().unwrap().matches,
                        completion_state.as_ref().unwrap().selected_index,
                    );
                }

                KeyCode::Enter => {
                    if let Some(state) = completion_state.as_ref()
                        && state.is_active_for(&editor)
                        && state.selected().is_some()
                    {
                        accept_completion(&mut editor, state);
                        clear_completions(&mut completion_state, &mut completion_lines);
                        redraw(prompt, &editor, theme);

                        continue;
                    }

                    clear_completion_lines(&mut completion_lines);

                    disable_raw_mode().unwrap();

                    println!();

                    return Some(editor.text());
                }
                _ => {}
            },

            _ => {}
        }
    }
}

fn refresh_input(prompt: &str, input: &[char], cursor_position: usize) {
    let mut stdout = io::stdout();

    stdout.execute(cursor::MoveToColumn(0)).unwrap();

    print!("\x1b[2K");

    let text: String = input.iter().collect();

    print!("{}{}", prompt, text);

    stdout
        .execute(cursor::MoveToColumn(
            width::cursor_column(prompt, &text, cursor_position) as u16,
        ))
        .unwrap();

    stdout.flush().unwrap();
}

fn move_cursor(prompt: &str, input: &[char], cursor_position: usize) {
    let mut stdout = io::stdout();
    let text: String = input.iter().collect();

    stdout
        .execute(cursor::MoveToColumn(
            width::cursor_column(prompt, &text, cursor_position) as u16,
        ))
        .unwrap();

    stdout.flush().unwrap();
}

fn clear_completion_lines(completion_lines: &mut usize) {
    if *completion_lines > 0 {
        render::clear_completion_lines(*completion_lines);
        *completion_lines = 0;
    }
}

fn render_completion_menu(
    prompt: &str,
    editor: &Editor,
    theme: &Theme,
    matches: &[String],
    selected_index: Option<usize>,
) -> usize {
    let highlighted = highlight::highlight(&editor.text());

    render::render_with_completions(
        prompt,
        &highlighted,
        editor.cursor(),
        theme,
        matches,
        selected_index,
    )
}

fn editor_word(editor: &Editor, start: usize, end: usize) -> String {
    editor
        .text()
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn accept_completion(editor: &mut Editor, state: &CompletionState) {
    let Some(selected) = state.selected() else {
        return;
    };

    editor.replace_range(state.word_start, state.word_end, selected);
}

fn redraw(prompt: &str, editor: &Editor, theme: &Theme) {
    let highlighted = highlight::highlight(&editor.text());

    render::render_highlighted(prompt, &highlighted, editor.cursor(), theme);
}

#[cfg(test)]
mod tests {
    use super::{CompletionState, accept_completion, editor_word};
    use crate::input::editor::Editor;

    fn editor_with_text(text: &str) -> Editor {
        let mut editor = Editor::new();
        editor.set_text(text);
        editor
    }

    #[test]
    fn selecting_completion_does_not_change_editor_text() {
        let editor = editor_with_text("ec");
        let mut state = CompletionState::new(
            vec!["echo".to_string(), "ecal".to_string()],
            0,
            editor.cursor(),
            editor_word(&editor, 0, editor.cursor()),
        );

        state.select_next();

        assert_eq!(state.selected_index, Some(0));
        assert_eq!(editor.text(), "ec");
    }

    #[test]
    fn completion_state_tracks_the_word_it_was_created_for() {
        let mut editor = editor_with_text("echo he");
        let state = CompletionState::new(
            vec!["hello".to_string()],
            5,
            editor.cursor(),
            editor_word(&editor, 5, editor.cursor()),
        );

        assert!(state.is_active_for(&editor));

        editor.insert('y');

        assert!(!state.is_active_for(&editor));
    }

    #[test]
    fn accepting_completion_replaces_the_tracked_word() {
        let mut editor = editor_with_text("echo he");
        let mut state = CompletionState::new(
            vec!["hello".to_string(), "help".to_string()],
            5,
            editor.cursor(),
            editor_word(&editor, 5, editor.cursor()),
        );

        state.select_next();

        accept_completion(&mut editor, &state);

        assert_eq!(editor.text(), "echo hello");
    }

    #[test]
    fn selecting_previous_completion_wraps_to_the_last_match() {
        let mut state = CompletionState::new(
            vec!["echo".to_string(), "ecal".to_string()],
            0,
            2,
            "ec".to_string(),
        );

        state.select_previous();

        assert_eq!(state.selected_index, Some(1));

        state.select_previous();

        assert_eq!(state.selected_index, Some(0));
    }
}
