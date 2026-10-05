use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

pub fn complete(input: &str, cursor: usize) -> Vec<String> {
    let before_cursor: String = input.chars().take(cursor).collect();

    let word_start = current_word_start(&before_cursor);

    let word = &before_cursor[word_start..];

    if is_command_position(&before_cursor[..word_start]) {
        complete_command(word)
    } else {
        complete_path(word)
    }
}

fn current_word_start(input: &str) -> usize {
    input
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0)
}

fn is_command_position(input: &str) -> bool {
    let mut command_position = true;
    let mut quote = None;
    let mut escape = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape {
            command_position = false;
            escape = false;
            continue;
        }

        if ch == '\\' && quote != Some('\'') {
            escape = true;
            continue;
        }

        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                command_position = false;
            }

            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            command_position = false;
            continue;
        }

        if ch.is_whitespace() {
            continue;
        }

        match ch {
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                }

                command_position = true;
            }

            ';' => {
                command_position = true;
            }

            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                }

                command_position = true;
            }

            '<' => {
                command_position = false;
            }

            '>' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                }

                command_position = false;
            }

            _ => {
                command_position = false;
            }
        }
    }

    command_position
}

fn complete_command(prefix: &str) -> Vec<String> {
    let mut commands = HashSet::new();

    let Some(path) = env::var_os("PATH") else {
        return Vec::new();
    };

    for directory in env::split_paths(&path) {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if !is_executable_file(&path) {
                continue;
            }

            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if name.starts_with(prefix) {
                commands.insert(name.to_string());
            }
        }
    }

    let mut commands: Vec<_> = commands.into_iter().collect();
    commands.sort();

    commands
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };

    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn complete_path(prefix: &str) -> Vec<String> {
    let expanded = expand_home(prefix);

    let path = Path::new(&expanded);

    let (directory, file_prefix) = if expanded.ends_with('/') {
        (path, "")
    } else {
        (
            path.parent().unwrap_or_else(|| Path::new(".")),
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(""),
        )
    };

    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut matches = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !name.starts_with(file_prefix) {
            continue;
        }

        let mut completion = build_completion(prefix, name);

        if path.is_dir() {
            completion.push('/');
        }

        matches.push(completion);
    }

    matches.sort();
    matches
}

fn build_completion(original: &str, filename: &str) -> String {
    match original.rfind('/') {
        Some(index) => {
            let prefix = &original[..=index];

            format!("{prefix}{filename}")
        }

        None => filename.to_string(),
    }
}

fn expand_home(path: &str) -> String {
    if path == "~" {
        return env::var("HOME").unwrap_or_else(|_| path.to_string());
    }

    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = env::var("HOME")
    {
        return PathBuf::from(home).join(rest).to_string_lossy().to_string();
    }

    path.to_string()
}

pub fn common_prefix(matches: &[String]) -> String {
    if matches.is_empty() {
        return String::new();
    }

    let mut prefix = matches[0].clone();

    for candidate in &matches[1..] {
        let common_length = prefix
            .chars()
            .zip(candidate.chars())
            .take_while(|(a, b)| a == b)
            .count();

        prefix = prefix.chars().take(common_length).collect();

        if prefix.is_empty() {
            break;
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::{common_prefix, current_word_start, is_command_position};

    #[test]
    fn detects_command_position_at_line_start() {
        assert!(is_command_position(""));
        assert!(is_command_position("   "));
    }

    #[test]
    fn detects_command_position_after_command_separators() {
        assert!(is_command_position("echo hi | "));
        assert!(is_command_position("false || "));
        assert!(is_command_position("true && "));
        assert!(is_command_position("pwd; "));
        assert!(is_command_position("sleep 1 & "));
    }

    #[test]
    fn detects_argument_position_after_command_words() {
        assert!(!is_command_position("echo "));
        assert!(!is_command_position("echo hi "));
        assert!(!is_command_position("cat < "));
        assert!(!is_command_position("echo > "));
    }

    #[test]
    fn ignores_operators_inside_quotes() {
        assert!(!is_command_position("echo \"|\" "));
        assert!(!is_command_position("echo ';' "));
    }

    #[test]
    fn finds_current_word_start_as_byte_index() {
        let input = "echo hello";
        let start = current_word_start(input);

        assert_eq!(&input[start..], "hello");
    }

    #[test]
    fn finds_common_prefix() {
        let matches = vec![
            "foobar".to_string(),
            "foobaz".to_string(),
            "fooqux".to_string(),
        ];

        assert_eq!(common_prefix(&matches), "foo");
    }
}
