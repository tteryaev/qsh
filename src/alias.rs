use std::collections::HashMap;

pub fn expand(input: &str, aliases: &HashMap<String, String>) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    let mut command_position = true;

    while index < chars.len() {
        let character = chars[index];

        if character.is_whitespace() {
            output.push(character);
            index += 1;
            continue;
        }

        if is_operator(character) {
            output.push(character);
            index += 1;

            if index < chars.len()
                && chars[index] == character
                && matches!(character, '|' | '&' | '>')
            {
                output.push(chars[index]);
                index += 1;
            }

            command_position = matches!(character, '|' | ';' | '&');
            continue;
        }

        let start = index;
        let mut quote = None;
        let mut escaped = false;

        while index < chars.len() {
            let current = chars[index];

            if escaped {
                escaped = false;
                index += 1;
                continue;
            }

            if current == '\\' && quote != Some('\'') {
                escaped = true;
                index += 1;
                continue;
            }

            if let Some(active_quote) = quote {
                if current == active_quote {
                    quote = None;
                }
                index += 1;
                continue;
            }

            if current == '\'' || current == '"' {
                quote = Some(current);
                index += 1;
                continue;
            }

            if current.is_whitespace() || is_operator(current) {
                break;
            }

            index += 1;
        }

        let word: String = chars[start..index].iter().collect();

        if command_position {
            if let Some(alias) = aliases.get(&word) {
                output.push_str(alias);
            } else {
                output.push_str(&word);
            }
            command_position = false;
        } else {
            output.push_str(&word);
        }
    }

    output
}

fn is_operator(character: char) -> bool {
    matches!(character, '|' | ';' | '&' | '<' | '>')
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::expand;

    fn aliases() -> HashMap<String, String> {
        HashMap::from([
            ("ll".to_string(), "ls -l".to_string()),
            ("gs".to_string(), "git status".to_string()),
        ])
    }

    #[test]
    fn expands_alias_at_the_start_of_a_command() {
        assert_eq!(expand("ll /tmp", &aliases()), "ls -l /tmp");
    }

    #[test]
    fn expands_aliases_after_command_separators() {
        let input = "ll; gs && ll || gs | ll";
        let expected = "ls -l; git status && ls -l || git status | ls -l";

        assert_eq!(expand(input, &aliases()), expected);
    }

    #[test]
    fn preserves_whitespace_and_quoted_operators() {
        let input = "ll  \"a;b\"; echo 'gs && ll'";
        let expected = "ls -l  \"a;b\"; echo 'gs && ll'";

        assert_eq!(expand(input, &aliases()), expected);
    }

    #[test]
    fn does_not_expand_arguments_with_alias_names() {
        assert_eq!(expand("echo ll; echo gs", &aliases()), "echo ll; echo gs");
    }
}
