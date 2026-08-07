use std::fs;
use std::path::{ Path, PathBuf };

use crate::error::{ ShellError, CommandError };

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    if args.is_empty() {
        return Err(ShellError::one("cp", "Missing operand"));
    }

    if args.len() < 2 {
        return Err(ShellError::one("cp", "Missing destination"));
    }

    let dest = Path::new(args.last().unwrap());
    let srcs = &args[..args.len() - 1];

    if srcs.len() > 1 && !dest.is_dir() {
        return Err(ShellError::one("cp", &format!("{}: Not a directory", dest.display())));
    }

    let mut errors: Vec<CommandError> = Vec::new();
    for src in srcs {
        if let Err(e) = copy_file(src, dest) {
            errors.push(e);
        }
    }

    if !errors.is_empty() {
        return Err(ShellError::Many(errors));
    }

    Ok(())
}

fn copy_file(src: &str, dest: &Path) -> Result<(), CommandError> {
    let src_path = Path::new(src);

    let dest_path = get_dest_path(src_path, dest)?;

    check_same_file(src, src_path, &dest_path)?;

    fs::copy(src_path, &dest_path).map_err(|e| CommandError::new_io("cp", &src, &e))?;

    Ok(())
}

fn get_dest_path(src: &Path, dest: &Path) -> Result<PathBuf, CommandError> {
    if dest.is_dir() {
        let filename = src
            .file_name()
            .ok_or_else(|| {
                CommandError::new("cp", &format!("{}: Invalid filename", src.display()))
            })?;

        Ok(dest.join(filename))
    } else {
        Ok(dest.to_path_buf())
    }
}

fn check_same_file(src: &str, src_path: &Path, dest_path: &Path) -> Result<(), CommandError> {
    if let Ok(src_real) = fs::canonicalize(src_path) {
        if let Ok(dest_real) = fs::canonicalize(dest_path) {
            if src_real == dest_real {
                return Err(
                    CommandError::new(
                        "cp",
                        &format!("'{}' and '{}' are the same file", src, dest_path.display())
                    )
                );
            }
        }
    }
    Ok(())
}
