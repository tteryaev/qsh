pub mod lua;
use std::collections::HashMap;
use std::path::PathBuf;
use lua::parse;

#[derive(Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub aliases: std::collections::HashMap<String, String>,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub prompt: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                prompt: "{current_directory}@{username} > ".to_string(),
            },

            aliases: HashMap::new(),

            plugins: Vec::new(),
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
