use crate::parser::lexer::Token;

#[derive(Debug)]
pub enum Highlight {

    Command(String),

    Argument(String),

    Operator(String),

    Error(String),

    Space(String),
}


pub fn highlight(
    input: &str,
) -> Vec<Highlight> {

    let tokens =
        crate::parser::tokenize(input);


    let mut result = Vec::new();

    let mut expect_command = true;


    for (index, token) in tokens.iter().enumerate() {

        if index > 0 {
            result.push(
                Highlight::Space(" ".to_string())
            );
        }

        match token {

            Token::Word { value, .. } => {

                if expect_command {

                    if is_command(&value) {

                        result.push(
                            Highlight::Command(value.to_string())
                        );

                    } else {

                        result.push(
                            Highlight::Error(value.to_string())
                        );
                    }


                    expect_command = false;

                } else {

                    result.push(
                        Highlight::Argument(value.to_string())
                    );
                }
            }


            Token::Pipe => {

                result.push(
                    Highlight::Operator(
                        "|".to_string()
                    )
                );

                expect_command = true;
            }


            Token::And => {

                result.push(
                    Highlight::Operator(
                        "&&".to_string()
                    )
                );

                expect_command = true;
            }


            Token::Or => {

                result.push(
                    Highlight::Operator(
                        "||".to_string()
                    )
                );

                expect_command = true;
            }


            Token::Semicolon => {

                result.push(
                    Highlight::Operator(
                        ";".to_string()
                    )
                );

                expect_command = true;
            }


            Token::RedirectOut => {

                result.push(
                    Highlight::Operator(
                        ">".to_string()
                    )
                );
            }


            Token::RedirectIn => {

                result.push(
                    Highlight::Operator(
                        "<".to_string()
                    )
                );
            }


            Token::AppendOut => {

                result.push(
                    Highlight::Operator(
                        ">>".to_string()
                    )
                );
            }


            Token::Background => {

                result.push(
                    Highlight::Operator(
                        "&".to_string()
                    )
                );
            }

        }
    }


    result
}


fn is_command(
    command: &str,
) -> bool {

    crate::builtin::exists(command)
        ||
        std::env::var_os("PATH")
            .unwrap_or_default()
            .to_string_lossy()
            .split(':')
            .any(|path| {

                std::path::Path::new(path)
                    .join(command)
                    .exists()
            })
}