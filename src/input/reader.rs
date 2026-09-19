use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

use crate::config::Theme;

use super::{
    editor::Editor,
    render,
    highlight,
    completion
};

use std::io::{self, Write};

fn clear_completions(
    completions: &mut Option<Vec<String>>,
    completion_index: &mut Option<usize>,
    completion_lines: &mut usize,
) {
    if *completion_lines > 0 {
        render::clear_completion_lines(
            *completion_lines,
        );

        *completion_lines = 0;
    }

    *completions = None;
    *completion_index = None;
}

pub fn read_input(
    prompt: &str,
    history: &[String],
    theme: &Theme,
) -> Option<String> {

    enable_raw_mode().unwrap();

    let mut editor = Editor::new();

    let mut history_index =
    history.len();

    let mut browsing_history = false;

    let mut completions: Option<Vec<String>> = None;
    let mut completion_lines = 0usize;

    let mut completion_index: Option<usize> = None;

    loop {

        match event::read().unwrap() {

            Event::Key(key) => {

                match key.code {
                    KeyCode::Char(c) => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        editor.insert(c);

                        browsing_history = false;

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Backspace => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        editor.backspace();

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Left => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        editor.move_left();

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Right => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        editor.move_right();

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Home => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        editor.move_home();

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::End => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        editor.move_end();

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );

                    }

                    KeyCode::Up => {
                        if history.is_empty() {
                            continue;
                        }

                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        browsing_history = true;

                        if history_index > 0 {
                            history_index -= 1;
                        }

                        editor.set_text(&history[history_index]);

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Down => {
                        if !browsing_history {
                            continue;
                        }

                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        if history_index + 1 < history.len() {
                            history_index += 1;

                            editor.set_text(&history[history_index]);
                        } else {
                            history_index = history.len();

                            editor.set_text("");
                        }

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Esc => {
                        clear_completions(
                            &mut completions,
                            &mut completion_index,
                            &mut completion_lines,
                        );

                        browsing_history = false;
                        history_index = history.len();

                        editor.set_text("");

                        redraw(
                            prompt,
                            &editor,
                            theme,
                        );
                    }

                    KeyCode::Tab => {
                        if let Some(matches) = completions.as_ref() {
                            if !matches.is_empty() {
                                let next_index = match completion_index {
                                    Some(index) => {
                                        (index + 1) % matches.len()
                                    }

                                    None => 0,
                                };

                                completion_index = Some(next_index);

                                // Remove old completion menu.
                                if completion_lines > 0 {
                                    render::clear_completion_lines(
                                        completion_lines,
                                    );

                                    completion_lines = 0;
                                }

                                editor.replace_current_word(
                                    &matches[next_index],
                                );

                                let highlighted =
                                highlight::highlight(
                                    &editor.text(),
                                );

                                completion_lines =
                                    render::render_with_completions(
                                        prompt,
                                        &highlighted,
                                        editor.cursor(),
                                        theme,
                                        matches,
                                    );
                            }

                            continue;
                        }

                        let input = editor.text();

                        let matches = completion::complete(
                            &input,
                            editor.cursor(),
                        );

                        if matches.is_empty() {
                            continue;
                        }

                        if matches.len() == 1 {
                            editor.replace_current_word(
                                &matches[0],
                            );

                            completion_index = None;

                            redraw(
                                prompt,
                                &editor,
                                theme,
                            );

                            continue;
                        }

                        let current_word_start =
                        editor.current_word_start();

                        let current_word_length =
                        editor.cursor() - current_word_start;

                        let prefix =
                        completion::common_prefix(&matches);

                        let prefix_length =
                        prefix.chars().count();

                        if prefix_length > current_word_length {
                            editor.replace_current_word(
                                &prefix,
                            );
                        }

                        completions = Some(matches);
                        completion_index = None;

                        let highlighted =
                        highlight::highlight(
                            &editor.text(),
                        );

                        completion_lines =
                            render::render_with_completions(
                                prompt,
                                &highlighted,
                                editor.cursor(),
                                theme,
                                completions
                                    .as_ref()
                                    .unwrap(),
                            );
                    }

                    KeyCode::Enter => {
                        if completion_lines > 0 {
                            render::clear_completion_lines(
                                completion_lines,
                            );

                            completion_lines = 0;
                        }

                        completions = None;
                        completion_index = None;

                        disable_raw_mode().unwrap();

                        println!();

                        return Some(editor.text());
                    }
                    _ => {}
                }

            }


            _ => {}
        }
    }
}



fn refresh_input(
    prompt: &str,
    input: &[char],
    cursor_position: usize,
) {

    let mut stdout =
    io::stdout();


    stdout
        .execute(
            cursor::MoveToColumn(0)
        )
        .unwrap();


    print!("\x1b[2K");


    let text: String =
    input.iter().collect();


    print!(
        "{}{}",
        prompt,
        text
    );


    stdout
        .execute(
            cursor::MoveToColumn(
                (prompt.len() + cursor_position)
                as u16
            )
        )
        .unwrap();


    stdout.flush().unwrap();
}



fn move_cursor(
    prompt: &str,
    input: &[char],
    cursor_position: usize,
) {

    let mut stdout =
    io::stdout();


    stdout
        .execute(
            cursor::MoveToColumn(
                (prompt.len() + cursor_position)
                as u16
            )
        )
        .unwrap();


    stdout.flush().unwrap();
}

fn redraw(
    prompt: &str,
    editor: &Editor,
    theme: &Theme,
) {

    let highlighted =
    highlight::highlight(
        &editor.text()
    );


    render::render_highlighted(
        prompt,
        &highlighted,
        editor.cursor(),
        theme,
    );
}
