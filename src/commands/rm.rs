use std::fs;
use std::path::Path;

use crate::error::{ ShellError, CommandError };

fn parse_args(args: &[String]) -> Result<(bool, Vec<String>), ShellError> {
    let mut recursive = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 {
            for ch in arg[1..].chars() {
                match ch {
                    'r' => {
                        recursive = true;
                    }
                    _ => {
                        return Err(ShellError::one("rm", &format!("invalid option: '{}'", arg)));
                    }
                }
            }
        } else {
            paths.push(arg.clone());
        }
    }

    Ok((recursive, paths))
}

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    let (recursive, paths) = parse_args(&args)?;

    if paths.is_empty() {
        return Err(ShellError::one("rm", "Missing operand"));
    }

    let mut errors: Vec<CommandError> = Vec::new();

    for path in &paths {
        let p = Path::new(path);
        if p.is_dir() {
            if recursive {
                if let Err(e) = fs::remove_dir_all(p) {
                    errors.push(CommandError::new_io("rm", &path, &e));
                }
            } else {
                errors.push(CommandError::new("rm", &format!("{}: Is a directory", path)));
            }
        } else {
            if let Err(e) = fs::remove_file(p) {
                errors.push(CommandError::new_io("rm", &path, &e));
            }
        }
    }

    if !errors.is_empty() {
        return Err(ShellError::Many(errors));
    }

    Ok(())
}
