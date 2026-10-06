use std::sync::{Arc, Mutex};

use mlua::{Lua, Result};

use super::{command::PluginCommand, context::PluginContext};

pub fn register(lua: &Lua, context: Arc<Mutex<PluginContext>>) -> Result<()> {
    let globals = lua.globals();

    let alias = lua.create_table()?;
    let command = lua.create_table()?;
    let event = lua.create_table()?;

    //
    // alias.add(name, command)
    //
    let alias_context = context.clone();

    let add_alias = lua.create_function(move |_, (name, value): (String, String)| {
        alias_context.lock().unwrap().aliases.insert(name, value);

        Ok(())
    })?;

    alias.set("add", add_alias)?;

    //
    // command.register(name, function)
    //
    let command_context = context.clone();

    let register_command =
        lua.create_function(move |lua, (name, callback): (String, mlua::Function)| {
            let key = lua.create_registry_value(callback)?;

            command_context
                .lock()
                .unwrap()
                .commands
                .push(PluginCommand {
                    name,
                    callback: key,
                });

            Ok(())
        })?;

    command.set("register", register_command)?;

    //
    // event.on_start(function)
    //
    let event_context = context.clone();

    let on_start = lua.create_function(move |lua, callback: mlua::Function| {
        let key = lua.create_registry_value(callback)?;

        event_context.lock().unwrap().events.on_start.push(key);

        Ok(())
    })?;

    event.set("on_start", on_start)?;

    let before_context = context.clone();

    let before_command = lua.create_function(move |lua, callback: mlua::Function| {
        let key = lua.create_registry_value(callback)?;

        before_context
            .lock()
            .unwrap()
            .events
            .before_command
            .push(key);

        Ok(())
    })?;

    let after_context = context.clone();

    let after_command = lua.create_function(move |lua, callback: mlua::Function| {
        let key = lua.create_registry_value(callback)?;

        after_context.lock().unwrap().events.after_command.push(key);

        Ok(())
    })?;

    event.set("after_command", after_command)?;

    event.set("before_command", before_command)?;

    //
    // Global API table
    //

    globals.set("alias", alias)?;

    globals.set("command", command)?;

    globals.set("event", event)?;

    Ok(())
}
