pub mod fuzzy;

use std::fs;
use std::path::PathBuf;

pub struct History {
    pub entries: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        let mut entries = Self::load();
        let original_len = entries.len();

        remove_consecutive_duplicates(&mut entries);

        if entries.len() != original_len {
            let _ = fs::write(Self::path(), entries.join("\n"));
        }

        Self { entries }
    }

    fn path() -> PathBuf {
        dirs::home_dir().unwrap().join(".qsh_history")
    }

    fn load() -> Vec<String> {
        match fs::read_to_string(Self::path()) {
            Ok(content) => content.lines().map(|x| x.to_string()).collect(),

            Err(_) => Vec::new(),
        }
    }

    pub fn add(&mut self, command: String) {
        let command = command.trim();

        if command.is_empty() {
            return;
        }

        if self.entries.last().is_some_and(|last| last == command) {
            return;
        }

        self.entries.push(command.to_string());

        let _ = fs::write(Self::path(), self.entries.join("\n"));
    }
}

fn remove_consecutive_duplicates(entries: &mut Vec<String>) {
    entries.dedup();
}

#[cfg(test)]
mod tests {
    use super::remove_consecutive_duplicates;

    #[test]
    fn removes_only_consecutive_duplicates() {
        let mut entries = vec![
            "clear".to_string(),
            "clear".to_string(),
            "fastfetch".to_string(),
            "fastfetch".to_string(),
            "firefox".to_string(),
            "firefox".to_string(),
            "firefox".to_string(),
            "clear".to_string(),
        ];

        remove_consecutive_duplicates(&mut entries);

        assert_eq!(
            entries,
            vec![
                "clear".to_string(),
                "fastfetch".to_string(),
                "firefox".to_string(),
                "clear".to_string(),
            ]
        );
    }
}
