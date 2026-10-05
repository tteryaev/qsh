use std::env;

pub fn execute(args: &[String]) -> bool {
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
