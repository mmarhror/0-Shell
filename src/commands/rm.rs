use std::io::{ Error, ErrorKind };
use std::fs;
use std::path::Path;

use crate::shell;

fn parse_args(args: &[String]) -> Result<(bool, Vec<String>), Error> {
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
                        return Err(
                            shell::format_error(
                                "rm",
                                ErrorKind::InvalidInput,
                                &format!("invalid option: '{}'", arg)
                            )
                        );
                    }
                }
            }
        } else {
            paths.push(arg.clone());
        }
    }

    Ok((recursive, paths))
}

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let (recursive, paths) = parse_args(&args)?;

    if paths.is_empty() {
        return Err(shell::format_error("rm", ErrorKind::InvalidInput, "Missing operand"));
    }

    for path in &paths {
        let p = Path::new(path);
        if p.is_dir() {
            if recursive {
                if let Err(e) = fs::remove_dir_all(p) {
                    handle_error(e, path);
                }
            } else {
                handle_error(Error::new(ErrorKind::Other, "is a directory"), path);
            }
        } else {
            if let Err(e) = fs::remove_file(p) {
                handle_error(e, path);
            }
        }
    }

    Ok(())
}

fn handle_error(e: Error, path: &str) {
    let msg = match e.kind() {
        ErrorKind::NotFound => format!("{}: No such file or directory", path),

        ErrorKind::PermissionDenied => format!("{}: Permission denied", path),

        _ => format!("'{}' Could not be removed", path),
    };

    eprintln!("{}", shell::format_error("rm", e.kind(), &msg));
}
