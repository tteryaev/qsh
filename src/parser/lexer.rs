pub enum Token {
    Word { value: String, expand: bool },

    Pipe,      // |
    Semicolon, // ;

    And, // &&
    Or,  // ||

    RedirectOut, // >
    RedirectIn,  // <
    AppendOut,   // >>

    Background, // &
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut current = String::new();

    let mut quote: Option<char> = None;

    let mut expand = true;

    let mut escape = false;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escape {
            current.push(c);

            escape = false;

            continue;
        }

        if c == '\\' && quote != Some('\'') {
            escape = true;

            continue;
        }

        // inside ""
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                current.push(c);
            }

            continue;
        }

        if c == '"' || c == '\'' {
            quote = Some(c);

            if c == '\'' {
                expand = false;
            }

            continue;
        }

        match c {
            ' ' | '\t' => {
                push_word(&mut tokens, &mut current, &mut expand);
            }

            '|' => {
                push_word(&mut tokens, &mut current, &mut expand);

                if chars.peek() == Some(&'|') {
                    chars.next();

                    tokens.push(Token::Or);
                } else {
                    tokens.push(Token::Pipe);
                }
            }

            ';' => {
                push_word(&mut tokens, &mut current, &mut expand);

                tokens.push(Token::Semicolon);
            }

            '>' => {
                push_word(&mut tokens, &mut current, &mut expand);

                if chars.peek() == Some(&'>') {
                    chars.next();

                    tokens.push(Token::AppendOut);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }

            '<' => {
                push_word(&mut tokens, &mut current, &mut expand);

                tokens.push(Token::RedirectIn);
            }

            '&' => {
                push_word(&mut tokens, &mut current, &mut expand);

                if chars.peek() == Some(&'&') {
                    chars.next();

                    tokens.push(Token::And);
                } else {
                    tokens.push(Token::Background);
                }
            }

            _ => {
                current.push(c);
            }
        }
    }

    if escape {
        current.push('\\');
    }

    push_word(&mut tokens, &mut current, &mut expand);

    tokens
}

fn push_word(tokens: &mut Vec<Token>, word: &mut String, expand: &mut bool) {
    if !word.is_empty() {
        tokens.push(Token::Word {
            value: word.clone(),
            expand: *expand,
        });

        word.clear();

        *expand = true;
    }
}
