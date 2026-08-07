use std::env;
use crate::error::ShellError;

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    let path = match args.len() {
        0 => env::var("HOME").map_err(|_| ShellError::one("cd", "HOME not set"))?,

        1 => args[0].clone(),
        _ => {
            return Err(ShellError::one("cd", "too many arguments"));
        }
    };

    env::set_current_dir(&path).map_err(|e| ShellError::one_io("cd", &path, &e))?;

    Ok(())
}
