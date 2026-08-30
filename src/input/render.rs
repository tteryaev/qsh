use std::io::{self, Write};

use crossterm::{
    cursor,
    ExecutableCommand,
};

use crate::{
    config::Theme,
    theme::color::Color,
};

use super::highlight::Highlight;



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