#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word {
        value: String,
        expand: bool,
        quoted: bool,
    },
    Pipe,
    Semicolon,
    Newline,
    And,
    Or,
    RedirectOut,
    RedirectIn,
    AppendOut,
    Background,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut word_started = false;
    let mut quote = None;
    let mut expand = true;
    let mut quoted = false;
    let mut escape = false;
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if escape {
            escape = false;
            if character != '\n' {
                current.push(character);
            }
            continue;
        }

        if character == '\\' && quote != Some('\'') {
            escape = true;
            quoted = true;
            word_started = true;
            continue;
        }

        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                current.push(character);
            }
            continue;
        }

        if character == '\'' || character == '"' {
            quote = Some(character);
            quoted = true;
            word_started = true;
            if character == '\'' {
                expand = false;
            }
            continue;
        }

        if character == '#' && !word_started {
            for comment_char in chars.by_ref() {
                if comment_char == '\n' {
                    tokens.push(Token::Newline);
                    break;
                }
            }
            continue;
        }

        match character {
            ' ' | '\t' | '\r' => push_word(
                &mut tokens,
                &mut current,
                &mut expand,
                &mut quoted,
                &mut word_started,
            ),
            '\n' => {
                push_word(
                    &mut tokens,
                    &mut current,
                    &mut expand,
                    &mut quoted,
                    &mut word_started,
                );
                tokens.push(Token::Newline);
            }
            '|' => {
                push_word(
                    &mut tokens,
                    &mut current,
                    &mut expand,
                    &mut quoted,
                    &mut word_started,
                );
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::Or);
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            ';' => {
                push_word(
                    &mut tokens,
                    &mut current,
                    &mut expand,
                    &mut quoted,
                    &mut word_started,
                );
                tokens.push(Token::Semicolon);
            }
            '>' => {
                push_word(
                    &mut tokens,
                    &mut current,
                    &mut expand,
                    &mut quoted,
                    &mut word_started,
                );
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::AppendOut);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }
            '<' => {
                push_word(
                    &mut tokens,
                    &mut current,
                    &mut expand,
                    &mut quoted,
                    &mut word_started,
                );
                tokens.push(Token::RedirectIn);
            }
            '&' => {
                push_word(
                    &mut tokens,
                    &mut current,
                    &mut expand,
                    &mut quoted,
                    &mut word_started,
                );
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::And);
                } else {
                    tokens.push(Token::Background);
                }
            }
            _ => {
                current.push(character);
                word_started = true;
            }
        }
    }

    if escape {
        current.push('\\');
    }
    push_word(
        &mut tokens,
        &mut current,
        &mut expand,
        &mut quoted,
        &mut word_started,
    );
    tokens
}

fn push_word(
    tokens: &mut Vec<Token>,
    word: &mut String,
    expand: &mut bool,
    quoted: &mut bool,
    word_started: &mut bool,
) {
    if !*word_started {
        return;
    }

    tokens.push(Token::Word {
        value: std::mem::take(word),
        expand: *expand,
        quoted: *quoted,
    });
    *expand = true;
    *quoted = false;
    *word_started = false;
}

pub fn has_line_continuation(input: &str) -> bool {
    let mut quote = None;
    let mut escape = false;

    for character in input.chars() {
        if escape {
            escape = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escape = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
        }
    }

    escape
}

pub fn has_unclosed_quote(input: &str) -> bool {
    let mut quote = None;
    let mut escape = false;

    for character in input.chars() {
        if escape {
            escape = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escape = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
        }
    }

    quote.is_some()
}
