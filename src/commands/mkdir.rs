use std::fs;

use crate::error::{ ShellError, CommandError };

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    if args.is_empty() {
        return Err(ShellError::one("mkdir", "Missing operand"));
    }

    let mut errors: Vec<CommandError> = Vec::new();

    for path in args {
        if let Err(e) = fs::create_dir(&path) {
            errors.push(CommandError::new_io("mkdir", &path, &e));
        };
    }

    if !errors.is_empty() {
        return Err(ShellError::Many(errors));
    }

    Ok(())
}
