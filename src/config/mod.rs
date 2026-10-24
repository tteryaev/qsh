pub mod lua;
use lua::parse;
use mlua::{Lua, RegistryKey};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Config {
    pub theme: Theme,
    pub aliases: std::collections::HashMap<String, String>,
    pub binds: HashMap<String, String>,
    pub custom_binds: Vec<CustomBind>,
    pub lua: Lua,
    pub plugins: Vec<String>,
}

pub struct CustomBind {
    pub key: String,
    pub callback: RegistryKey,
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
    pub history: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub prompt: String,
    pub prompt_colors: HashMap<String, String>,
    pub widgets: HashMap<String, String>,
    pub syntax: SyntaxTheme,
    pub completion: CompletionTheme,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                prompt: "{current_directory}@{username} > ".to_string(),
                prompt_colors: HashMap::new(),
                widgets: HashMap::new(),
                syntax: SyntaxTheme::default(),
                completion: CompletionTheme::default(),
            },

            aliases: HashMap::new(),
            binds: HashMap::from([
                ("break_line".to_string(), "ctrl+c".to_string()),
                ("clear_line".to_string(), "ctrl+u".to_string()),
                ("exit".to_string(), "ctrl+d".to_string()),
                ("clear_screen".to_string(), "ctrl+l".to_string()),
                ("move_start".to_string(), "ctrl+a".to_string()),
                ("move_end".to_string(), "ctrl+e".to_string()),
                ("delete_word_backward".to_string(), "ctrl+w".to_string()),
                ("delete_to_end".to_string(), "ctrl+k".to_string()),
                ("history_search".to_string(), "ctrl+r".to_string()),
            ]),
            custom_binds: Vec::new(),
            lua: Lua::new(),

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
            history: None,
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
