use std::collections::HashMap;

use super::{
    Config,
    Theme,
    SyntaxTheme,
};

use mlua::Lua;

pub fn parse(code: &str) -> Result<Config, mlua::Error> {
    let lua = Lua::new();

    lua.load(code).exec()?;

    let globals = lua.globals();

    let theme_table =
        globals
            .get::<mlua::Table>("theme")
            .ok();

    let prompt = match globals.get::<mlua::Table>("theme") {
        Ok(theme) => theme
            .get::<String>("prompt")
            .unwrap_or_else(|_| Config::default().theme.prompt),

        Err(_) => Config::default().theme.prompt,
    };

    let syntax = match &theme_table {

        Some(theme) => {

            match theme.get::<mlua::Table>("syntax") {

                Ok(syntax) => SyntaxTheme {

                    command: syntax
                        .get::<String>("command")
                        .ok(),

                    argument: syntax
                        .get::<String>("argument")
                        .ok(),

                    error: syntax
                        .get::<String>("error")
                        .ok(),

                    operator: syntax
                        .get::<String>("operator")
                        .ok(),
                },


                Err(_) => SyntaxTheme::default(),
            }
        }


        None => SyntaxTheme::default(),
    };

    let theme = Theme {
        prompt,
        syntax,
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
