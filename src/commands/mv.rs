use std::fs;
use std::path::{ Path, PathBuf };

use crate::error::{ ShellError, CommandError };

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    if args.is_empty() {
        return Err(ShellError::one("mv", "Missing operand"));
    }

    if args.len() < 2 {
        return Err(ShellError::one("mv", "Missing destination"));
    }

    let dest = Path::new(args.last().unwrap());
    let srcs = &args[..args.len() - 1];

    if srcs.len() > 1 && !dest.is_dir() {
        return Err(ShellError::one("mv", &format!("{}: Not a directory", dest.display())));
    }

    let mut errors: Vec<CommandError> = Vec::new();

    for src in srcs {
        let src_path = Path::new(src);

        let dest_path = match get_dest_path(src_path, dest) {
            Ok(p) => p,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };

        if let Err(e) = fs::rename(src_path, dest_path) {
            errors.push(CommandError::new_io("mv", &src, &e));
        }
    }

    if !errors.is_empty() {
        return Err(ShellError::Many(errors));
    }

    Ok(())
}

fn get_dest_path(src: &Path, dest: &Path) -> Result<PathBuf, CommandError> {
    if dest.is_dir() {
        let filename = src
            .file_name()
            .ok_or_else(|| {
                CommandError::new("mv", &format!("{}: Invalid filename", src.display()))
            })?;

        Ok(dest.join(filename))
    } else {
        Ok(dest.to_path_buf())
    }
}
