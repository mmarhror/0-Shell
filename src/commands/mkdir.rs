use std::io::{ Error, ErrorKind };
use std::fs::create_dir;

use crate::shell;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() {
        return Err(shell::format_error("mkdir", ErrorKind::InvalidInput, "Missing operand"));
    }

    for path in args {
        if let Err(e) = create_dir(&path) {
            let msg = match e.kind() {
                ErrorKind::AlreadyExists => format!("{}: File exists", path),

                ErrorKind::NotFound => format!("{}: No such file or directory", path),

                ErrorKind::PermissionDenied => format!("{}: Permission denied", path),

                _ => format!("{}: {}", path, e),
            };

            return Err(shell::format_error("mkdir", e.kind(), &msg));
        }
    }

    Ok(())
}
