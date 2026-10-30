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
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    render_highlighted_with_suggestion(
        prompt,
        parts,
        cursor_position,
        theme,
        None,
        input_start_column,
        previous_cursor_row,
    );
}

pub fn render_highlighted_with_suggestion(
    _prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    suggestion: Option<&str>,
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    let mut stdout = io::stdout();
    begin_synchronized_update();

    let text = parts_text(parts);
    begin_input_redraw(&mut stdout, input_start_column, previous_cursor_row);

    for part in parts {
        let (text, color) = match part {
            Highlight::Command(text) => (text, Color::from_option(&theme.syntax.command)),

            Highlight::Argument(text) => (text, Color::from_option(&theme.syntax.argument)),

            Highlight::Operator(text) => (text, Color::from_option(&theme.syntax.operator)),

            Highlight::Error(text) => (text, Color::from_option(&theme.syntax.error)),

            Highlight::Space(text) => (text, Color::Default),
        };

        print!("{}", color.ansi());
        print_input_text(text);
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
    clear_rendered_tail(&mut stdout);

    // Restore cursor position
    restore_input_cursor(
        &mut stdout,
        &text,
        cursor_position,
        suggestion.unwrap_or(""),
        input_start_column,
    );

    stdout.flush().unwrap();
    end_synchronized_update();
    stdout.flush().unwrap();
}

pub fn render_history_search(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    query: &str,
    candidate: Option<&str>,
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    render_highlighted(
        prompt,
        parts,
        cursor_position,
        theme,
        input_start_column,
        previous_cursor_row,
    );

    let mut stdout = io::stdout();
    let (column, _) = cursor::position().unwrap_or((0, 0));

    print!("\r\n\x1b[90msearch command: {}", query);
    if let Some(candidate) = candidate {
        print!(" → {}", candidate);
    }
    print!("\x1b[39m\x1b[K");

    stdout.execute(cursor::MoveUp(1)).unwrap();
    stdout.execute(cursor::MoveToColumn(column)).unwrap();
    stdout.flush().unwrap();
}

pub fn render_plain(
    _prompt: &str,
    text: &str,
    cursor_position: usize,
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    let mut stdout = io::stdout();
    begin_synchronized_update();

    begin_input_redraw(&mut stdout, input_start_column, previous_cursor_row);
    print_input_text(text);
    clear_rendered_tail(&mut stdout);

    restore_input_cursor(&mut stdout, text, cursor_position, "", input_start_column);

    stdout.flush().unwrap();
    end_synchronized_update();
    stdout.flush().unwrap();
}

pub fn render_completions(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    completions: &[String],
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    render_highlighted(
        prompt,
        parts,
        cursor_position,
        theme,
        input_start_column,
        previous_cursor_row,
    );

    print!("\r\n");

    for (index, completion) in completions.iter().enumerate() {
        if index > 0 {
            print!("    ");
        }

        print!("{}", completion);
    }

    print!("\r\n");

    // Redraw the input after the completion list.
    render_highlighted(
        prompt,
        parts,
        cursor_position,
        theme,
        input_start_column,
        previous_cursor_row,
    );
}

pub fn render_with_completions(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    completions: &[String],
    selected_index: Option<usize>,
    input_start_column: usize,
    previous_cursor_row: usize,
) -> usize {
    let mut stdout = io::stdout();
    begin_synchronized_update();

    render_input(
        prompt,
        parts,
        cursor_position,
        theme,
        input_start_column,
        previous_cursor_row,
    );

    let (terminal_columns, terminal_height) = terminal::size().unwrap_or((80, 24));

    let (_, cursor_row) = cursor::position().unwrap_or((0, 0));

    // Keep one line as a safety margin so that
    // the completion menu never causes terminal scrolling.
    let available_rows = terminal_height.saturating_sub(cursor_row + 2) as usize;

    if available_rows == 0 {
        end_synchronized_update();
        stdout.flush().unwrap();
        return 0;
    }

    let column_width = completions
        .iter()
        .map(|completion| width::display_width(completion))
        .max()
        .unwrap_or(1)
        + 4;

    let columns = std::cmp::max(1, terminal_columns as usize / column_width);

    let total_rows = completions.len().div_ceil(columns);

    // Keep the selected row visible when the menu is taller than the screen.
    let selected_row = selected_index.map(|index| index / columns);
    let (scroll_offset, rows) = completion_window(total_rows, available_rows, selected_row);

    if rows == 0 {
        end_synchronized_update();
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

    let text_before_cursor = parts_text(parts)
        .chars()
        .take(cursor_position)
        .collect::<String>();
    let cursor_column =
        visual_position_from_column(&text_before_cursor, terminal_width(), input_start_column).1;
    stdout
        .execute(cursor::MoveToColumn(cursor_column as u16))
        .unwrap();

    stdout.flush().unwrap();
    end_synchronized_update();
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

fn render_input(
    _prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    let mut stdout = io::stdout();

    let text = parts_text(parts);
    begin_input_redraw(&mut stdout, input_start_column, previous_cursor_row);

    for part in parts {
        match part {
            Highlight::Command(text) => {
                print!("{}", Color::from_option(&theme.syntax.command).ansi());
                print_input_text(text);
            }

            Highlight::Argument(text) => {
                print!("{}", Color::from_option(&theme.syntax.argument).ansi());
                print_input_text(text);
            }

            Highlight::Operator(text) => {
                print!("{}", Color::from_option(&theme.syntax.operator).ansi());
                print_input_text(text);
            }

            Highlight::Error(text) => {
                print!("{}", Color::from_option(&theme.syntax.error).ansi());
                print_input_text(text);
            }

            Highlight::Space(text) => {
                print_input_text(text);
            }
        }
    }

    print!("{}", Color::Default.ansi());
    clear_rendered_tail(&mut stdout);

    restore_input_cursor(&mut stdout, &text, cursor_position, "", input_start_column);

    stdout.flush().unwrap();
}

fn begin_input_redraw(
    stdout: &mut io::Stdout,
    input_start_column: usize,
    previous_cursor_row: usize,
) {
    if previous_cursor_row > 0 {
        stdout
            .execute(cursor::MoveUp(
                previous_cursor_row.min(u16::MAX as usize) as u16
            ))
            .unwrap();
    }

    stdout
        .execute(cursor::MoveToColumn(input_start_column as u16))
        .unwrap();
}

fn clear_rendered_tail(stdout: &mut io::Stdout) {
    stdout.execute(Clear(ClearType::FromCursorDown)).unwrap();
}

fn begin_synchronized_update() {
    print!("\x1b[?2026h");
}

fn end_synchronized_update() {
    print!("\x1b[?2026l");
}

pub fn input_cursor_row(text: &str, cursor_position: usize, input_start_column: usize) -> usize {
    let text_before_cursor = text.chars().take(cursor_position).collect::<String>();
    visual_position_from_column(&text_before_cursor, terminal_width(), input_start_column).0
}

fn print_input_text(text: &str) {
    print!("{}", normalize_newlines(text));
}

pub fn render_prompt(prompt: &str) {
    print!("{}", normalize_newlines(prompt));
    io::stdout().flush().unwrap();
}

fn restore_input_cursor(
    stdout: &mut io::Stdout,
    text: &str,
    cursor_position: usize,
    rendered_suffix: &str,
    input_start_column: usize,
) {
    let text_before_cursor = text.chars().take(cursor_position).collect::<String>();
    let rendered = format!("{}{}", text, rendered_suffix);
    let (cursor_row, cursor_column) =
        visual_position_from_column(&text_before_cursor, terminal_width(), input_start_column);
    let (rendered_row, _) =
        visual_position_from_column(&rendered, terminal_width(), input_start_column);

    let rows_up = rendered_row.saturating_sub(cursor_row);
    if rows_up > 0 {
        stdout
            .execute(cursor::MoveUp(rows_up.min(u16::MAX as usize) as u16))
            .unwrap();
    }
    stdout
        .execute(cursor::MoveToColumn(cursor_column as u16))
        .unwrap();
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

fn terminal_width() -> usize {
    terminal::size()
        .map(|(columns, _)| columns.max(1) as usize)
        .unwrap_or(80)
}

/// Calculates the visual row and column after writing text in a terminal.
/// The row count includes both explicit newlines and automatic line wrapping.
fn visual_position_from_column(
    text: &str,
    terminal_width: usize,
    start_column: usize,
) -> (usize, usize) {
    let mut row = 0;
    let mut column = start_column.min(terminal_width.saturating_sub(1));
    let mut pending_wrap = false;

    for (index, line) in normalize_newlines(text).split('\n').enumerate() {
        if index > 0 {
            row += 1;
            column = 0;
            pending_wrap = false;
        }

        let line_width = width::display_width(line.strip_suffix('\r').unwrap_or(line));
        if line_width == 0 {
            continue;
        }

        if pending_wrap {
            row += 1;
            column = 0;
            pending_wrap = false;
        }

        let end = column + line_width;
        row += end.saturating_sub(1) / terminal_width;
        let end_column = end % terminal_width;
        if end_column == 0 {
            column = terminal_width - 1;
            pending_wrap = true;
        } else {
            column = end_column;
        }
    }

    (row, column)
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
