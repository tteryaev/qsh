use std::io::{self, Write};

use crossterm::{
    ExecutableCommand, cursor,
    terminal::{self, Clear, ClearType},
};

use crate::{config::Theme, theme::color::Color};

use crate::input::{render, width};

use super::highlight::Highlight;

fn clear_completions(
    completions: &mut Option<Vec<String>>,
    completion_index: &mut Option<usize>,
    completion_lines: &mut usize,
) {
    if *completion_lines > 0 {
        render::clear_completion_lines(*completion_lines);

        *completion_lines = 0;
    }

    *completions = None;
    *completion_index = None;
}

pub fn clear_completion_lines(lines: usize) {
    if lines == 0 {
        return;
    }

    let mut stdout = io::stdout();

    // Move to the first completion line.
    stdout.execute(cursor::MoveDown(1)).unwrap();

    // The input cursor may be in the middle of the line. Start clearing from
    // the beginning so no part of the first completion row is left behind.
    stdout.execute(cursor::MoveToColumn(0)).unwrap();

    // Clear everything below the input line.
    stdout.execute(Clear(ClearType::FromCursorDown)).unwrap();

    // Return to the input line.
    stdout.execute(cursor::MoveUp(1)).unwrap();

    stdout.execute(cursor::MoveToColumn(0)).unwrap();

    stdout.flush().unwrap();
}

fn completion_window(
    total_rows: usize,
    available_rows: usize,
    selected_row: Option<usize>,
) -> (usize, usize) {
    let rows = total_rows.min(available_rows);

    if rows == 0 {
        return (0, 0);
    }

    let max_offset = total_rows - rows;
    let selected_offset = selected_row
        .map(|row| row.min(total_rows - 1).saturating_sub(rows - 1))
        .unwrap_or(0);

    (selected_offset.min(max_offset), rows)
}

pub fn render_highlighted(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
) {
    render_highlighted_with_suggestion(prompt, parts, cursor_position, theme, None);
}

pub fn render_highlighted_with_suggestion(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    suggestion: Option<&str>,
) {
    let mut stdout = io::stdout();

    begin_prompt_redraw(&mut stdout, prompt);
    print_prompt(prompt);

    for part in parts {
        let (text, color) = match part {
            Highlight::Command(text) => (text, Color::from_option(&theme.syntax.command)),

            Highlight::Argument(text) => (text, Color::from_option(&theme.syntax.argument)),

            Highlight::Operator(text) => (text, Color::from_option(&theme.syntax.operator)),

            Highlight::Error(text) => (text, Color::from_option(&theme.syntax.error)),

            Highlight::Space(text) => (text, Color::Default),
        };

        print!("{}{}", color.ansi(), text);
    }

    if let Some(suggestion) = suggestion {
        let color = match theme.completion.history {
            Some(_) => Color::from_option(&theme.completion.history).ansi(),
            None => "\x1b[90m".to_string(),
        };

        print!("{}{}{}", color, suggestion, Color::Default.ansi());
    }

    // Reset terminal color
    print!("{}", Color::Default.ansi());

    // Restore cursor position
    stdout
        .execute(cursor::MoveToColumn(input_cursor_column(
            prompt,
            &parts_text(parts),
            cursor_position,
        ) as u16))
        .unwrap();

    stdout.flush().unwrap();
}

pub fn render_plain(prompt: &str, text: &str, cursor_position: usize) {
    let mut stdout = io::stdout();

    begin_prompt_redraw(&mut stdout, prompt);
    print_prompt(prompt);
    print!("{}", text);

    stdout
        .execute(cursor::MoveToColumn(
            input_cursor_column(prompt, text, cursor_position) as u16,
        ))
        .unwrap();

    stdout.flush().unwrap();
}

pub fn render_completions(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    completions: &[String],
) {
    render_highlighted(prompt, parts, cursor_position, theme);

    print!("\r\n");

    for (index, completion) in completions.iter().enumerate() {
        if index > 0 {
            print!("    ");
        }

        print!("{}", completion);
    }

    print!("\r\n");

    // Redraw the input after the completion list.
    render_highlighted(prompt, parts, cursor_position, theme);
}

pub fn render_with_completions(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    completions: &[String],
    selected_index: Option<usize>,
) -> usize {
    let mut stdout = io::stdout();

    render_input(prompt, parts, cursor_position, theme);

    let (terminal_width, terminal_height) = terminal::size().unwrap_or((80, 24));

    let (_, cursor_row) = cursor::position().unwrap_or((0, 0));

    // Keep one line as a safety margin so that
    // the completion menu never causes terminal scrolling.
    let available_rows = terminal_height.saturating_sub(cursor_row + 2) as usize;

    if available_rows == 0 {
        stdout.flush().unwrap();
        return 0;
    }

    let column_width = completions
        .iter()
        .map(|completion| width::display_width(completion))
        .max()
        .unwrap_or(1)
        + 4;

    let columns = std::cmp::max(1, terminal_width as usize / column_width);

    let total_rows = completions.len().div_ceil(columns);

    // Keep the selected row visible when the menu is taller than the screen.
    let selected_row = selected_index.map(|index| index / columns);
    let (scroll_offset, rows) = completion_window(total_rows, available_rows, selected_row);

    if rows == 0 {
        stdout.flush().unwrap();
        return 0;
    }

    print!("\r\n");

    for row in 0..rows {
        for column in 0..columns {
            let index = (scroll_offset + row) * columns + column;

            if index >= completions.len() {
                break;
            }

            let completion = &completions[index];

            if selected_index == Some(index) {
                if theme.completion.selected.is_some() {
                    let color = Color::from_option(&theme.completion.selected);
                    print!("{}{}{}", color.ansi(), completion, Color::Default.ansi());
                } else {
                    print!("\x1b[7m{}\x1b[27m", completion);
                }
            } else {
                let color = Color::from_option(&theme.completion.unselected);
                print!("{}{}{}", color.ansi(), completion, Color::Default.ansi());
            }

            if column + 1 < columns {
                let width = width::display_width(completion);

                let padding = column_width.saturating_sub(width);

                print!("{:padding$}", "");
            }
        }

        if row + 1 < rows {
            print!("\r\n");
        }
    }

    // Return to the input line.
    stdout.execute(cursor::MoveUp(rows as u16)).unwrap();

    stdout
        .execute(cursor::MoveToColumn(width::cursor_column(
            prompt,
            &parts_text(parts),
            cursor_position,
        ) as u16))
        .unwrap();

    stdout.flush().unwrap();

    rows
}

#[cfg(test)]
mod tests {
    use super::completion_window;

    #[test]
    fn completion_window_starts_at_the_first_row_without_selection() {
        assert_eq!(completion_window(6, 3, None), (0, 3));
    }

    #[test]
    fn completion_window_scrolls_to_the_selected_row() {
        assert_eq!(completion_window(6, 3, Some(4)), (2, 3));
        assert_eq!(completion_window(6, 3, Some(5)), (3, 3));
    }

    #[test]
    fn completion_window_does_not_scroll_past_the_end() {
        assert_eq!(completion_window(2, 5, Some(1)), (0, 2));
    }
}

fn render_input(prompt: &str, parts: &[Highlight], cursor_position: usize, theme: &Theme) {
    let mut stdout = io::stdout();

    begin_prompt_redraw(&mut stdout, prompt);
    print_prompt(prompt);

    for part in parts {
        match part {
            Highlight::Command(text) => {
                print!(
                    "{}{}",
                    Color::from_option(&theme.syntax.command).ansi(),
                    text
                );
            }

            Highlight::Argument(text) => {
                print!(
                    "{}{}",
                    Color::from_option(&theme.syntax.argument).ansi(),
                    text
                );
            }

            Highlight::Operator(text) => {
                print!(
                    "{}{}",
                    Color::from_option(&theme.syntax.operator).ansi(),
                    text
                );
            }

            Highlight::Error(text) => {
                print!("{}{}", Color::from_option(&theme.syntax.error).ansi(), text);
            }

            Highlight::Space(text) => {
                print!("{}", text);
            }
        }
    }

    print!("{}", Color::Default.ansi());

    stdout
        .execute(cursor::MoveToColumn(input_cursor_column(
            prompt,
            &parts_text(parts),
            cursor_position,
        ) as u16))
        .unwrap();

    stdout.flush().unwrap();
}

fn begin_prompt_redraw(stdout: &mut io::Stdout, prompt: &str) {
    let prompt_lines = prompt.bytes().filter(|byte| *byte == b'\n').count();

    if prompt_lines > 0 {
        stdout
            .execute(cursor::MoveUp(prompt_lines.min(u16::MAX as usize) as u16))
            .unwrap();
    }

    stdout.execute(cursor::MoveToColumn(0)).unwrap();
    stdout.execute(Clear(ClearType::FromCursorDown)).unwrap();
}

fn print_prompt(prompt: &str) {
    // Raw mode does not translate LF to CRLF, so normalize prompt newlines
    // to keep every prompt line anchored at column zero.
    let normalized = prompt.replace("\r\n", "\n").replace('\n', "\r\n");
    print!("{}", normalized);
}

fn input_cursor_column(prompt: &str, text: &str, cursor_position: usize) -> usize {
    let final_prompt_line = prompt.rsplit('\n').next().unwrap_or(prompt);
    let final_prompt_line = final_prompt_line
        .strip_suffix('\r')
        .unwrap_or(final_prompt_line);
    width::cursor_column(final_prompt_line, text, cursor_position)
}

fn parts_text(parts: &[Highlight]) -> String {
    parts
        .iter()
        .map(|part| match part {
            Highlight::Command(text)
            | Highlight::Argument(text)
            | Highlight::Operator(text)
            | Highlight::Error(text)
            | Highlight::Space(text) => text.as_str(),
        })
        .collect()
}
