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
    highlight
};

use std::io::{self, Write};


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



    loop {

        match event::read().unwrap() {

            Event::Key(key) => {

                match key.code {

                    KeyCode::Char(c) => {

                        editor.insert(c);

                        browsing_history = false;

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Backspace => {

                        editor.backspace();

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Left => {

                        editor.move_left();

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Right => {

                        editor.move_right();

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Home => {

                        editor.move_home();

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::End => {

                        editor.move_end();

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Up => {

                        if history.is_empty() {
                            continue;
                        }


                        browsing_history = true;


                        if history_index > 0 {
                            history_index -= 1;
                        }


                        editor.set_text(
                            &history[history_index]
                        );


                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Down => {

                        if !browsing_history {
                            continue;
                        }


                        if history_index + 1 < history.len() {

                            history_index += 1;


                            editor.set_text(
                                &history[history_index]
                            );

                        } else {

                            history_index = history.len();

                            editor.set_text("");
                        }


                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Esc => {

                        browsing_history = false;

                        history_index = history.len();


                        editor.set_text("");

                        redraw(prompt, &editor, theme);
                    }


                    KeyCode::Enter => {

                        disable_raw_mode().unwrap();

                        println!();


                        return Some(
                            editor.text()
                        );
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
