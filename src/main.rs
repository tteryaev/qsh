mod alias;
mod builtin;
mod config;
mod executor;
mod expand;
mod history;
mod input;
mod parser;
mod plugins;
mod theme;
mod version;

use std::env;
use std::io::{self, Write};
use std::path::Path;

use history::History;
use input::reader::InputResult;

use whoami;

fn get_username() -> String {
    whoami::username().expect("qsh: Failed to get username")
}

fn format_user_path(full_path: &Path) -> String {
    if let Some(home) = env::home_dir() {
        if let Ok(relative) = full_path.strip_prefix(&home) {
            if relative.as_os_str().is_empty() {
                return "~".to_string();
            }

            return format!("~/{}", relative.display());
        }
    }

    full_path.display().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ignore_sigint_in_shell();

    let config = config::load_config();

    let mut history = History::new();

    let mut plugin_manager = plugins::manager::PluginManager::new();

    plugin_manager.load_plugins(&config.plugins);

    plugin_manager
        .context
        .lock()
        .unwrap()
        .events
        .run_on_start(&plugin_manager.lua);

    let plugin_aliases = plugin_manager.context.lock().unwrap().aliases.clone();
    let mut aliases = config.aliases.clone();

    aliases.extend(plugin_aliases);

    loop {
        let username = get_username();

        let path = format_user_path(&env::current_dir()?);

        let prompt = theme::format_greeting(
            &config.theme.prompt,
            &username,
            &path,
            &config.theme.widgets,
            &config.theme.prompt_colors,
        );

        print!("{}", prompt);

        io::stdout().flush()?;
        let raw_input = match input::read_input(
            &prompt,
            &history.entries,
            &config.theme,
            &aliases,
            &config.binds,
            &config.custom_binds,
            &config.lua,
        ) {
            InputResult::Submit(input) => input,
            InputResult::BreakLine => continue,
            InputResult::Exit => break,
        };

        let raw_input = raw_input.trim();

        history.add(raw_input.to_string());

        if raw_input.is_empty() {
            continue;
        }
        if raw_input == "exit" {
            break;
        }

        let input = alias::expand(raw_input, &aliases);

        //parser

        let tokens = parser::tokenize(&input);

        match parser::parse(&tokens) {
            Ok(Some(ast)) => {
                executor::execute(ast, &plugin_manager);
            }
            Ok(None) => {}
            Err(parser::parser::ParseError::Incomplete) => {
                eprintln!("qsh: incomplete input");
            }
            Err(parser::parser::ParseError::Syntax(error)) => {
                eprintln!("qsh: syntax error: {error}");
            }
        }
    }

    Ok(())
}

fn ignore_sigint_in_shell() {
    // External children restore SIGINT to its default disposition before exec.
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
    }
}
