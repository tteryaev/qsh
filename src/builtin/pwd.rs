use std::io::Write;


pub fn execute<W: Write>(
    output: &mut W,
) -> bool {

    match std::env::current_dir() {

        Ok(path) => {

            writeln!(
                output,
                "{}",
                path.display()
            )
                .is_ok()
        }


        Err(error) => {

            eprintln!(
                "pwd: {}",
                error
            );

            false
        }
    }
}