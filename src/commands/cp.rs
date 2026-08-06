use std::fs;
use std::io::{ Error, ErrorKind };
use std::path::{ Path, PathBuf };

use crate::shell;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() {
        return Err(shell::format_error("cp", ErrorKind::InvalidInput, "Missing operand"));
    }

    if args.len() < 2 {
        return Err(shell::format_error("cp", ErrorKind::InvalidInput, "Missing destination"));
    }

    let dest = Path::new(args.last().unwrap());
    let srcs = &args[..args.len() - 1];

    if srcs.len() > 1 && !dest.is_dir() {
        return Err(
            shell::format_error(
                "cp",
                ErrorKind::InvalidInput,
                &format!("{}: Not a directory", dest.display())
            )
        );
    }

    for src in srcs {
        copy_file(src, &dest)?;
    }

    Ok(())
}

fn copy_file(src: &str, dest: &Path) -> Result<(), Error> {
    let src_path = Path::new(src);

    if src_path.is_dir() {
        return Err(
            shell::format_error("cp", ErrorKind::IsADirectory, &format!("{}: Is a directory", src))
        );
    }

    let dest_path = get_dest_path(src_path, dest)?;

    check_same_file(src, src_path, &dest_path)?;

    fs::copy(src_path, &dest_path).map_err(|e| {
        let msg = match e.kind() {
            ErrorKind::NotFound => format!("{}: No such file or directory", src),

            ErrorKind::PermissionDenied => format!("{}: Permission denied", src),

            ErrorKind::IsADirectory => format!("{}: Is a directory", src),
            _ => format!("{}: {}", src, e),
        };
        shell::format_error("cp", e.kind(), &msg)
    })?;

    Ok(())
}

fn get_dest_path(src: &Path, dest: &Path) -> Result<PathBuf, Error> {
    if dest.is_dir() {
        let filename = src
            .file_name()
            .ok_or_else(|| {
                shell::format_error(
                    "cp",
                    ErrorKind::InvalidInput,
                    &format!("{}: Invalid filename", src.display())
                )
            })?;

        Ok(dest.join(filename))
    } else {
        Ok(dest.to_path_buf())
    }
}

fn check_same_file(src: &str, src_path: &Path, dest_path: &Path) -> Result<(), Error> {
    if let Ok(src_real) = fs::canonicalize(src_path) {
        if let Ok(dest_real) = fs::canonicalize(dest_path) {
            if src_real == dest_real {
                return Err(
                    shell::format_error(
                        "cp",
                        ErrorKind::InvalidInput,
                        &format!("'{}' and '{}' are the same file", src, dest_path.display())
                    )
                );
            }
        }
    }
    Ok(())
}
