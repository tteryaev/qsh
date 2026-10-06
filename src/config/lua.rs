use std::collections::HashMap;

use super::{CompletionTheme, Config, SyntaxTheme, Theme};

use mlua::Lua;

pub fn parse(code: &str) -> Result<Config, mlua::Error> {
    let lua = Lua::new();

    lua.load(code).exec()?;

    let globals = lua.globals();

    let theme_table = globals.get::<mlua::Table>("theme").ok();

    let prompt = match globals.get::<mlua::Table>("theme") {
        Ok(theme) => theme
            .get::<String>("prompt")
            .unwrap_or_else(|_| Config::default().theme.prompt),

        Err(_) => Config::default().theme.prompt,
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
