use std::env;

use crate::error::ShellError;

pub fn run() -> Result<(), ShellError> {
    let cwd = env::current_dir().map_err(|e| ShellError::one("pwd", &e.to_string()))?;

    println!("{}", cwd.display());

    Ok(())
}
