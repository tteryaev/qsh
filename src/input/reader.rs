use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

use std::io::{self, Write};


pub fn read_input(
    prompt: &str,
    history: &[String],
) -> Option<String> {

    enable_raw_mode().unwrap();


    let mut input: Vec<char> = Vec::new();

    let mut cursor_position = 0;


    let mut history_index =
        history.len();

    let mut browsing_history = false;



    loop {

        match event::read().unwrap() {

            Event::Key(key) => {

                match key.code {


                    KeyCode::Char(c) => {

                        input.insert(
                            cursor_position,
                            c,
                        );


                        cursor_position += 1;

                        browsing_history = false;


                        refresh_input(
                            prompt,
                            &input,
                            cursor_position,
                        );
                    }



                    KeyCode::Backspace => {

                        if cursor_position > 0 {

                            cursor_position -= 1;


                            input.remove(
                                cursor_position
                            );


                            refresh_input(
                                prompt,
                                &input,
                                cursor_position,
                            );
                        }
                    }



                    KeyCode::Left => {

                        if cursor_position > 0 {

                            cursor_position -= 1;


                            move_cursor(
                                prompt,
                                &input,
                                cursor_position,
                            );
                        }
                    }



                    KeyCode::Right => {

                        if cursor_position < input.len() {

                            cursor_position += 1;


                            move_cursor(
                                prompt,
                                &input,
                                cursor_position,
                            );
                        }
                    }



                    KeyCode::Home => {

                        cursor_position = 0;


                        move_cursor(
                            prompt,
                            &input,
                            cursor_position,
                        );
                    }



                    KeyCode::End => {

                        cursor_position =
                            input.len();


                        move_cursor(
                            prompt,
                            &input,
                            cursor_position,
                        );
                    }



                    KeyCode::Up => {

                        if history.is_empty() {
                            continue;
                        }


                        browsing_history = true;


                        if history_index > 0 {

                            history_index -= 1;
                        }


                        input =
                            history[history_index]
                                .chars()
                                .collect();


                        cursor_position =
                            input.len();


                        refresh_input(
                            prompt,
                            &input,
                            cursor_position,
                        );
                    }



                    KeyCode::Down => {

                        if !browsing_history {
                            continue;
                        }


                        if history_index + 1 < history.len() {

                            history_index += 1;


                            input =
                                history[history_index]
                                    .chars()
                                    .collect();


                            cursor_position =
                                input.len();


                        } else {

                            history_index =
                                history.len();


                            input.clear();

                            cursor_position = 0;
                        }


                        refresh_input(
                            prompt,
                            &input,
                            cursor_position,
                        );
                    }



                    KeyCode::Esc => {

                        browsing_history = false;

                        history_index =
                            history.len();


                        input.clear();

                        cursor_position = 0;


                        refresh_input(
                            prompt,
                            &input,
                            cursor_position,
                        );
                    }



                    KeyCode::Enter => {

                        disable_raw_mode()
                            .unwrap();


                        println!();


                        return Some(
                            input
                                .iter()
                                .collect()
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
