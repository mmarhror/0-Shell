use std::io::{ self, Error, ErrorKind, Write };
use std::fs;

use crate::shell;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() {
        return Err(shell::format_error("cat", ErrorKind::InvalidInput, "Missing operand"));
    }

    let mut out = io::stdout();

    for path in args {
        match fs::read(&path) {
            Ok(content) => {
                out.write_all(&content)?;
                out.flush()?;
            }
            Err(e) => {
                let msg = match e.kind() {
                    ErrorKind::NotFound => format!("{}: No such file or directory", path),

                    ErrorKind::PermissionDenied => format!("{}: Permission denied", path),

                    ErrorKind::IsADirectory => format!("{}: Is a directory", path),

                    _ => format!("{}: {}", path, e),
                };

                return Err(shell::format_error("cat", e.kind(), &msg));
            }
        }
    }

    Ok(())
}
