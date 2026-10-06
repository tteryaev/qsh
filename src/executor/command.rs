use std::process::Command;

use crate::builtin;
use crate::parser::Command as ShellCommand;
use crate::plugins::manager::PluginManager;

use super::output::Output;
use super::redirect;

pub fn execute(command: ShellCommand, plugins: &PluginManager) -> bool {
    let allowed = plugins
        .context
        .lock()
        .unwrap()
        .events
        .run_before_command(&plugins.lua, command.program.clone());

    if !allowed {
        return false;
    }

    // Plugin commands have priority over builtins.
    if let Some(result) = plugins.execute_command(&command.program, command.args.clone()) {
        return result;
    }

    /* Builtin commands are executed inside qsh,
    so their output must be handled separately. */

    if builtin::exists(&command.program) {
        let mut output = match Output::from_redirect(command.stdout) {
            Ok(output) => output,

            Err(error) => {
                eprintln!("qsh: {}", error);

                return false;
            }
        };

        return builtin::execute(&command.program, &command.args, &mut output).unwrap_or(false);
    }

    // External command.
    let mut process = Command::new(&command.program);

    process.args(&command.args);

    redirect::apply_stdin(&mut process, command.stdin);

    redirect::apply_stdout(&mut process, command.stdout);

    match process.status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("qsh: exited with {}", status);
            }

            status.success()
        }

        Err(error) => {
            eprintln!("qsh: {}: {}", command.program, error);

            false
        }
    }
}
