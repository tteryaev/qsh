pub mod lua;
use lua::parse;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub aliases: std::collections::HashMap<String, String>,
    pub plugins: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub command: Option<String>,
    pub argument: Option<String>,
    pub error: Option<String>,
    pub operator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompletionTheme {
    pub selected: Option<String>,
    pub unselected: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub prompt: String,
    pub syntax: SyntaxTheme,
    pub completion: CompletionTheme,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                prompt: "{current_directory}@{username} > ".to_string(),
                syntax: SyntaxTheme::default(),
                completion: CompletionTheme::default(),
            },

            aliases: HashMap::new(),

            plugins: Vec::new(),
        }
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            command: None,
            argument: None,
            error: None,
            operator: None,
        }
    }
}

impl Default for CompletionTheme {
    fn default() -> Self {
        Self {
            selected: None,
            unselected: None,
        }
    }
}

pub fn load_config() -> Config {
    let path = config_path();

    let code: String =
        std::fs::read_to_string(&path).unwrap_or_else(|_| include_str!("default.lua").to_string());

    parse(&code).unwrap_or_else(|error| {
        eprintln!("qsh config error: {}", error);

        Config::default()
    })
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("qsh")
        .join("config.lua")
}
