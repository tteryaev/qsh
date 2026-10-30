pub mod command;
pub mod output;
pub mod pipeline;
pub mod redirect;

use crate::parser::Statement;

use crate::plugins;
use crate::plugins::manager::PluginManager;

pub fn execute(statement: Statement, plugins: &PluginManager) -> bool {
    match statement {
        Statement::Command(command) => command::execute(command, plugins),

        Statement::Pipeline(pipeline) => pipeline::execute(pipeline.commands),

        Statement::Sequence(commands) => {
            let mut success = true;

            for command in commands {
                success = execute(command, plugins);
            }

            success
        }

        Statement::And(left, right) => {
            if execute(*left, plugins) {
                execute(*right, plugins)
            } else {
                false
            }
        }

        Statement::Or(left, right) => {
            if execute(*left, plugins) {
                true
            } else {
                execute(*right, plugins)
            }
        }

        Statement::If {
            branches,
            else_branch,
        } => {
            for (condition, body) in branches {
                if execute(condition, plugins) {
                    return execute(body, plugins);
                }
            }

            else_branch
                .map(|branch| execute(*branch, plugins))
                .unwrap_or(false)
        }
    }
}
