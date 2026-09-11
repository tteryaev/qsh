mod cd;
mod echo;
mod pwd;


use std::io::Write;


pub fn exists(
    command: &str,
) -> bool {

    matches!(
        command,
        "cd"
            | "echo"
            | "pwd"
            | "exit"
    )
}



pub fn execute<W: Write>(
    command: &str,
    args: &[String],
    output: &mut W,
) -> Option<bool> {

    match command {

        "echo" => {

            Some(
                echo::execute(
                    args,
                    output,
                )
            )
        }


        "cd" => {

            Some(
                cd::execute(args)
            )
        }


        "pwd" => {

            Some(
                pwd::execute(output)
            )
        }


        "exit" => {

            Some(true)
        }


        _ => None,
    }
}