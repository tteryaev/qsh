use super::redirect;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::parser::Command as ShellCommand;

pub fn execute(commands: Vec<ShellCommand>) -> bool {
    let mut children = Vec::new();

    let mut previous_stdout = None;

    for (index, command) in commands.iter().enumerate() {
        let mut process = Command::new(&command.program);

        process.args(&command.args);

        #[cfg(unix)]
        unsafe {
            process.pre_exec(|| {
                libc::signal(libc::SIGINT, libc::SIG_DFL);
                Ok(())
            });
        }

        if let Some(stdout) = previous_stdout.take() {
            process.stdin(Stdio::from(stdout));
        }

        if index < commands.len() - 1 {
            process.stdout(Stdio::piped());
        } else {
            redirect::apply_stdout(&mut process, command.stdout.clone());
        }

        match process.spawn() {
            Ok(mut child) => {
                previous_stdout = child.stdout.take();

                children.push(child);
            }

            Err(error) => {
                eprintln!("qsh: {}: {}", command.program, error);

                return false;
            }
        }
    }
    let mut success = true;

    for mut child in children {
        match child.wait() {
            Ok(status) => {
                if !status.success() {
                    success = false;
                }
            }

            Err(_) => {
                success = false;
            }
        }
    }

    success
}
