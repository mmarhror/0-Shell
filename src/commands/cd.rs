use std::io::{ Error, ErrorKind };
use std::env;

use crate::shell;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let path = match args.len() {
        0 =>
            match env::var("HOME") {
                Ok(home) => home,
                Err(_) => {
                    return Err(shell::format_error("cd", ErrorKind::NotFound, "HOME not set"));
                }
            }
        1 => args[0].clone(),
        _ => {
            return Err(shell::format_error("cd", ErrorKind::InvalidInput, "too many arguments"));
        }
    };

    if let Err(e) = env::set_current_dir(&path) {
        let msg = match e.kind() {
            ErrorKind::NotFound => format!("{}: No such file or directory", path),
            ErrorKind::PermissionDenied => format!("{}: Permission denied", path),
            ErrorKind::NotADirectory => format!("{}: Not a directory", path),
            _ => format!("{}: {}", path, e),
        };

        return Err(shell::format_error("cd", e.kind(), &msg));
    }

    Ok(())
}
