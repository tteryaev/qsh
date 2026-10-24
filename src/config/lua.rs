use std::collections::HashMap;

use super::{CompletionTheme, Config, CustomBind, SyntaxTheme, Theme};

use mlua::{Lua, Value};

pub fn parse(code: &str) -> Result<Config, mlua::Error> {
    let lua = Lua::new();
    let defaults = Config::default();

    lua.load(code).exec()?;

    let globals = lua.globals();

    let theme_table = globals.get::<mlua::Table>("theme").ok();

    let prompt = match globals.get::<mlua::Table>("theme") {
        Ok(theme) => theme
            .get::<String>("prompt")
            .unwrap_or_else(|_| defaults.theme.prompt.clone()),

        Err(_) => defaults.theme.prompt.clone(),
    };

    let prompt_colors = theme_table
        .as_ref()
        .and_then(|theme| theme.get::<mlua::Table>("prompt_colors").ok())
        .map(read_string_table)
        .unwrap_or_default();

    let widgets = theme_table
        .as_ref()
        .and_then(|theme| theme.get::<mlua::Table>("widgets").ok())
        .map(read_string_table)
        .unwrap_or_default();

    let syntax = match &theme_table {
        Some(theme) => match theme.get::<mlua::Table>("syntax") {
            Ok(syntax) => SyntaxTheme {
                command: syntax.get::<String>("command").ok(),

                argument: syntax.get::<String>("argument").ok(),

                error: syntax.get::<String>("error").ok(),

                operator: syntax.get::<String>("operator").ok(),
            },

            Err(_) => SyntaxTheme::default(),
        },

        None => SyntaxTheme::default(),
    };

    let completion = match &theme_table {
        Some(theme) => match theme.get::<mlua::Table>("completion") {
            Ok(completion) => CompletionTheme {
                selected: completion.get::<String>("selected").ok(),
                unselected: completion.get::<String>("unselected").ok(),
                history: completion.get::<String>("history").ok(),
            },
            Err(_) => CompletionTheme::default(),
        },
        None => CompletionTheme::default(),
    };

    let theme = Theme {
        prompt,
        prompt_colors,
        widgets,
        syntax,
        completion,
    };

    let mut aliases = HashMap::new();

    if let Ok(table) = globals.get::<mlua::Table>("aliases") {
        for pair in table.pairs::<String, String>() {
            if let Ok((name, command)) = pair {
                aliases.insert(name, command);
            }
        }
    }

    let mut binds = defaults.binds.clone();
    let mut custom_binds = Vec::new();
    if let Ok(table) = globals.get::<mlua::Table>("binds") {
        for pair in table.pairs::<String, Value>() {
            let Ok((name, value)) = pair else {
                continue;
            };

            match value {
                Value::String(key) => {
                    binds.insert(name, key.to_str()?.to_string());
                }
                Value::Function(callback) => {
                    let callback = lua.create_registry_value(callback)?;
                    custom_binds.push(CustomBind {
                        key: name,
                        callback,
                    });
                }
                _ => {}
            }
        }
    }

    let mut plugins = Vec::new();

    if let Ok(table) = globals.get::<mlua::Table>("plugins") {
        if let Ok(enabled) = table.get::<mlua::Table>("enabled") {
            for plugin in enabled.sequence_values::<String>() {
                if let Ok(plugin) = plugin {
                    plugins.push(plugin);
                }
            }
        }
    }

    Ok(Config {
        theme,

        aliases,

        binds,

        custom_binds,

        lua,

        plugins,
    })
}

fn read_string_table(table: mlua::Table) -> HashMap<String, String> {
    table
        .pairs::<String, String>()
        .filter_map(Result::ok)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_completion_colors() {
        let config = parse(
            r##"
                theme = {
                    completion = {
                        selected = "#ffffff",
                        unselected = "#888888",
                        history = "#666666",
                    },
                }
            "##,
        )
        .unwrap();

        assert_eq!(config.theme.completion.selected.as_deref(), Some("#ffffff"));
        assert_eq!(
            config.theme.completion.unselected.as_deref(),
            Some("#888888")
        );
        assert_eq!(config.theme.completion.history.as_deref(), Some("#666666"));
    }
}
