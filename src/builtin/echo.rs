use std::io::Write;

pub fn execute<W: Write>(args: &[String], output: &mut W) -> bool {
    writeln!(output, "{}", args.join(" ")).is_ok()
}
