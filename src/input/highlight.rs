#[derive(Debug, PartialEq, Eq)]
pub enum Highlight {
    Command(String),
    Argument(String),
    Operator(String),
    Error(String),
    Space(String),
}

/// Highlights the input without changing the text that will be rendered.
///
/// The parser discards quotes, escapes and repeated whitespace. That is useful
/// for execution, but not for an editor, so highlighting scans source text and
/// only uses the decoded word to decide its color.
pub fn highlight(input: &str) -> Vec<Highlight> {
    highlight_with_aliases(input, &HashMap::new())
}

pub fn highlight_with_aliases(input: &str, aliases: &HashMap<String, String>) -> Vec<Highlight> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut expect_command = true;
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if escaped {
            word.push(character);
            escaped = false;
            continue;
        }

        if character == '\\' && quote != Some('\'') {
            word.push(character);
            escaped = true;
            continue;
        }

        if let Some(active_quote) = quote {
            word.push(character);
            if character == active_quote {
                quote = None;
            }
            continue;
        }

        if character == '\'' || character == '"' {
            word.push(character);
            quote = Some(character);
            continue;
        }

        if character.is_whitespace() {
            push_word(&mut result, &mut word, &mut expect_command, aliases);

            let mut spaces = character.to_string();
            while chars.peek().is_some_and(|next| next.is_whitespace()) {
                spaces.push(chars.next().unwrap());
            }
            result.push(Highlight::Space(spaces));
            continue;
        }

        let operator = match character {
            '|' if chars.peek() == Some(&'|') => {
                chars.next();
                Some("||")
            }
            '&' if chars.peek() == Some(&'&') => {
                chars.next();
                Some("&&")
            }
            '>' if chars.peek() == Some(&'>') => {
                chars.next();
                Some(">>")
            }
            '|' | ';' | '>' | '<' | '&' => Some(match character {
                '|' => "|",
                ';' => ";",
                '>' => ">",
                '<' => "<",
                '&' => "&",
                _ => unreachable!(),
            }),
            _ => None,
        };

        if let Some(operator) = operator {
            push_word(&mut result, &mut word, &mut expect_command, aliases);
            result.push(Highlight::Operator(operator.to_string()));
            expect_command = matches!(operator, "|" | "||" | ";" | "&" | "&&");
        } else {
            word.push(character);
        }
    }

    push_word(&mut result, &mut word, &mut expect_command, aliases);
    result
}

fn push_word(
    result: &mut Vec<Highlight>,
    word: &mut String,
    expect_command: &mut bool,
    aliases: &HashMap<String, String>,
) {
    if word.is_empty() {
        return;
    }

    let source = std::mem::take(word);

    if *expect_command {
        if is_command(&unquote(&source), aliases) {
            result.push(Highlight::Command(source));
        } else {
            result.push(Highlight::Error(source));
        }
        *expect_command = false;
    } else {
        result.push(Highlight::Argument(source));
    }
}

fn unquote(source: &str) -> String {
    let mut value = String::new();
    let mut quote = None;
    let mut escaped = false;

    for character in source.chars() {
        if escaped {
            value.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                value.push(character);
            }
        } else if character == '\'' || character == '"' {
            quote = Some(character);
        } else {
            value.push(character);
        }
    }

    if escaped {
        value.push('\\');
    }
    value
}

fn is_command(command: &str, aliases: &HashMap<String, String>) -> bool {
    aliases.contains_key(command)
        || crate::builtin::exists(command)
        || std::env::var_os("PATH")
            .unwrap_or_default()
            .to_string_lossy()
            .split(':')
            .any(|path| std::path::Path::new(path).join(command).exists())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Highlight, highlight, highlight_with_aliases};

    fn rendered_text(parts: &[Highlight]) -> String {
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

    #[test]
    fn preserves_exact_whitespace() {
        let input = "echo    hello\tworld";
        assert_eq!(rendered_text(&highlight(input)), input);
    }

    #[test]
    fn preserves_quotes_and_escapes() {
        let input = r#"echo "hello world" 'quoted' hello\ world"#;
        assert_eq!(rendered_text(&highlight(input)), input);
    }

    #[test]
    fn does_not_treat_operators_inside_quotes_as_operators() {
        let parts = highlight(r#"echo "a|b && c; d""#);
        assert!(matches!(parts[2], Highlight::Argument(_)));
        assert!(
            !parts
                .iter()
                .any(|part| matches!(part, Highlight::Operator(_)))
        );
    }

    #[test]
    fn recognizes_commands_after_separators() {
        let parts = highlight("echo ok | pwd");
        assert!(matches!(parts[0], Highlight::Command(_)));
        assert!(matches!(parts[6], Highlight::Command(_)));
    }

    #[test]
    fn recognizes_aliases_as_commands() {
        let aliases = HashMap::from([(String::from("ll"), String::from("ls -l"))]);
        let parts = highlight_with_aliases("ll /tmp", &aliases);

        assert!(matches!(parts[0], Highlight::Command(_)));
    }
}
use std::collections::HashMap;
