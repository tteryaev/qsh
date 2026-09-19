use std::io::{self, Write};

use crossterm::{
    cursor,
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};

use crate::{
    config::Theme, input, theme::color::Color,
};

use crate::input::render;

use super::highlight::Highlight;

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

pub fn clear_completion_lines(lines: usize) {
    if lines == 0 {
        return;
    }

    let mut stdout = io::stdout();

    // Move to the first completion line.
    stdout
        .execute(cursor::MoveDown(1))
        .unwrap();

    // Clear everything below the input line.
    stdout
        .execute(Clear(ClearType::FromCursorDown))
        .unwrap();

    // Return to the input line.
    stdout
        .execute(cursor::MoveUp(1))
        .unwrap();

    stdout
        .execute(cursor::MoveToColumn(0))
        .unwrap();

    stdout.flush().unwrap();
}

pub fn render_highlighted(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
) {

    let mut stdout = io::stdout();


    // Move to the beginning of the line and clear it
    stdout
        .execute(
            cursor::MoveToColumn(0)
        )
        .unwrap();


    print!("\x1b[2K");


    print!("{}", prompt);

    for part in parts {

        let (text, color) =
            match part {

                Highlight::Command(text) => {

                    (
                        text,
                        Color::from_option(
                            &theme.syntax.command
                        )
                    )
                }


                Highlight::Argument(text) => {

                    (
                        text,
                        Color::from_option(
                            &theme.syntax.argument
                        )
                    )
                }


                Highlight::Operator(text) => {

                    (
                        text,
                        Color::from_option(
                            &theme.syntax.operator
                        )
                    )
                }


                Highlight::Error(text) => {

                    (
                        text,
                        Color::from_option(
                            &theme.syntax.error
                        )
                    )
                }

                Highlight::Space(text) => {

                    (
                        text,
                        Color::Default
                    )
                }

            };


        print!(
            "{}{}",
            color.ansi(),
            text
        );
    }



    // Reset terminal color
    print!(
        "{}",
        Color::Default.ansi()
    );



    // Restore cursor position
    stdout
        .execute(
            cursor::MoveToColumn(
                (
                    prompt.len()
                        + cursor_position
                ) as u16
            )
        )
        .unwrap();


    stdout.flush().unwrap();
}



pub fn render_plain(
    prompt: &str,
    text: &str,
    cursor_position: usize,
) {

    let mut stdout = io::stdout();


    // Move to the beginning of the line and clear it
    stdout
        .execute(
            cursor::MoveToColumn(0)
        )
        .unwrap();


    print!("\x1b[2K");


    print!(
        "{}{}",
        prompt,
        text
    );


    stdout
        .execute(
            cursor::MoveToColumn(
                (
                    prompt.len()
                        + cursor_position
                ) as u16
            )
        )
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
    render_highlighted(
        prompt,
        parts,
        cursor_position,
        theme,
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
    );
}

pub fn render_with_completions(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
    completions: &[String],
) -> usize {
    let mut stdout = io::stdout();

    render_input(
        prompt,
        parts,
        cursor_position,
        theme,
    );

    let (terminal_width, terminal_height) =
        terminal::size().unwrap_or((80, 24));

    let (_, cursor_row) =
        cursor::position().unwrap_or((0, 0));

    // Keep one line as a safety margin so that
    // the completion menu never causes terminal scrolling.
    let available_rows = terminal_height
        .saturating_sub(cursor_row + 2) as usize;

    if available_rows == 0 {
        stdout.flush().unwrap();
        return 0;
    }

    let column_width = completions
        .iter()
        .map(|completion| completion.chars().count())
        .max()
        .unwrap_or(1)
        + 4;

    let columns = std::cmp::max(
        1,
        terminal_width as usize / column_width,
    );

    let total_rows =
        completions.len().div_ceil(columns);

    // Only show rows that fit on the screen.
    let rows = total_rows.min(available_rows);

    if rows == 0 {
        stdout.flush().unwrap();
        return 0;
    }

    print!("\r\n");

    for row in 0..rows {
        for column in 0..columns {
            let index = row * columns + column;

            if index >= completions.len() {
                break;
            }

            let completion = &completions[index];

            print!("{}", completion);

            if column + 1 < columns {
                let width = completion.chars().count();

                let padding =
                    column_width.saturating_sub(width);

                print!("{:width$}", "");
            }
        }

        if row + 1 < rows {
            print!("\r\n");
        }
    }

    // Return to the input line.
    stdout
        .execute(cursor::MoveUp(rows as u16))
        .unwrap();

    stdout
        .execute(cursor::MoveToColumn(
            (prompt.len() + cursor_position) as u16,
        ))
        .unwrap();

    stdout.flush().unwrap();

    rows
}

fn render_input(
    prompt: &str,
    parts: &[Highlight],
    cursor_position: usize,
    theme: &Theme,
) {
    let mut stdout = io::stdout();

    stdout
        .execute(cursor::MoveToColumn(0))
        .unwrap();

    print!("\x1b[2K");
    print!("{}", prompt);

    for part in parts {
        match part {
            Highlight::Command(text) => {
                print!(
                    "{}{}",
                    Color::from_option(
                        &theme.syntax.command
                    )
                        .ansi(),
                    text
                );
            }

            Highlight::Argument(text) => {
                print!(
                    "{}{}",
                    Color::from_option(
                        &theme.syntax.argument
                    )
                        .ansi(),
                    text
                );
            }

            Highlight::Operator(text) => {
                print!(
                    "{}{}",
                    Color::from_option(
                        &theme.syntax.operator
                    )
                        .ansi(),
                    text
                );
            }

            Highlight::Error(text) => {
                print!(
                    "{}{}",
                    Color::from_option(
                        &theme.syntax.error
                    )
                        .ansi(),
                    text
                );
            }

            Highlight::Space(text) => {
                print!("{}", text);
            }
        }
    }

    print!("{}", Color::Default.ansi());

    stdout
        .execute(
            cursor::MoveToColumn(
                (prompt.len() + cursor_position) as u16
            )
        )
        .unwrap();

    stdout.flush().unwrap();
}


