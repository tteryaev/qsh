use std::env;
use std::io::Write;

pub fn exists(command: &str) -> bool {
    matches!(
        command,
        "cd"
            | "echo"
            | "pwd"
            | "exit"
    )
}

pub fn execute(command: &str, args: &[String]) -> Option<bool> {
    match command {
        "echo" => {
            echo(args);

            Some(true)
        }

        "cd" => Some(cd(args)),

        "pwd" => {
            pwd();

            Some(true)
        }

        "exit" => Some(true),

        _ => None,
    }
}



fn echo(args: &[String]) {
    println!("{}", args.join(" "));
}
fn cd(args: &[String]) -> bool {
    let path = if args.is_empty() {
        match env::home_dir() {
            Some(home) => home,
            None => {
                eprintln!("cd: cannot find home directory");
                return false;
            }
        }
    } else if args.len() == 1 {
        args[0].clone().into()
    } else {
        eprintln!("cd: too many arguments");
        return false;
    };

    match env::set_current_dir(path) {
        Ok(_) => true,

        Err(error) => {
            eprintln!("cd: {}", error);

            false
        }
    }
}

fn pwd() {
    match env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
        }

        Err(error) => {
            eprintln!("pwd: {}", error);
        }
    }
}
