use std::io::{ Error, ErrorKind };
use std::fs;
use std::path::Path;

use crate::shell;

fn handle_error(e: Error, path: &str) {
    let msg = match e.kind() {
        ErrorKind::NotFound => format!("{}: No such file or directory", path),

        ErrorKind::PermissionDenied => format!("{}: Permission denied", path),

        _ => format!("'{}' Could not be removed", path),
    };

    eprintln!("{}", shell::format_error("rm", e.kind(), &msg));
}

pub fn run(args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() || (args.len() == 1 && args[0] == "-r") {
        return Err(shell::format_error("rm", ErrorKind::InvalidInput, "Missing operand"));
    }

    let recursive = args[0] == "-r";
    let paths = if recursive { &args[1..] } else { &args[..] };

    for path in paths {
        let p = Path::new(path);
        if p.is_dir() {
            if recursive {
                if let Err(e) = fs::remove_dir_all(p) {
                    handle_error(e, path);
                    continue;
                }
            } else {
                eprintln!("rm: {}: is a directory", path);
            }
        } else {
            if let Err(e) = fs::remove_file(p) {
                handle_error(e, path);
            }
        }
    }

    Ok(())
}
