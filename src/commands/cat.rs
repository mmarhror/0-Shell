use std::io::{ Error, ErrorKind };
use std::fs;

use crate::shell;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() {
        return Err(shell::format_error("cat", ErrorKind::InvalidInput, "Missing operand"));
    }

    for path in args {
        match fs::read_to_string(&path) {
            Ok(content) => {
                print!("{content}");
            }
            Err(e) => {
                let msg = match e.kind() {
                    ErrorKind::NotFound => format!("{}: No such file or directory", path),

                    ErrorKind::PermissionDenied => format!("{}: Permission denied", path),

                    ErrorKind::InvalidData => format!("{}: Invalid UTF-8", path),

                    ErrorKind::IsADirectory => format!("{}: Is a directory", path),

                    _ => format!("{}: {}", path, e),
                };

                return Err(shell::format_error("cat", e.kind(), &msg));
            }
        }
    }

    Ok(())
}
