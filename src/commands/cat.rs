use std::fs;
use std::io::{ self, Write };

use crate::error::{ CommandError, ShellError };

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    if args.is_empty() {
        return Err(ShellError::one("cat", "Missing operand"));
    }

    let mut out = io::stdout();
    let mut errors: Vec<CommandError> = Vec::new();

    for path in args {
        match fs::read(&path) {
            Ok(content) => {
                if let Err(e) = out.write_all(&content) {
                    errors.push(CommandError::new_io("cat", &path, &e));
                    continue;
                }
                if let Err(e) = out.flush() {
                    errors.push(CommandError::new_io("cat", &path, &e));
                    continue;
                }
            }
            Err(e) => {
                errors.push(CommandError::new_io("cat", &path, &e));
            }
        }
    }

    if !errors.is_empty() {
        return Err(ShellError::Many(errors));
    }

    Ok(())
}
