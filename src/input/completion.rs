use std::{
    collections::HashSet,
    env,
    fs,
    path::{Path, PathBuf},
};

pub fn complete(input: &str, cursor: usize) -> Vec<String> {
    let before_cursor: String = input.chars().take(cursor).collect();

    let word_start = before_cursor
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0);

    let word = &before_cursor[word_start..];

    let is_command = before_cursor[..word_start].trim().is_empty();

    if is_command {
        complete_command(word)
    } else {
        complete_path(word)
    }
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

            if !path.is_file() {
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

    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home)
                .join(rest)
                .to_string_lossy()
                .to_string();
        }
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

        prefix = prefix
            .chars()
            .take(common_length)
            .collect();

        if prefix.is_empty() {
            break;
        }
    }

    prefix
}
