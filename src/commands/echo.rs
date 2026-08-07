use crate::error::ShellError;

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    println!("{}", args.join(" "));
    Ok(())
}
