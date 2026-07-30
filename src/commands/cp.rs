use std::fs;
use std::io::{ Error, ErrorKind };
use std::path::Path;

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
        let src_path = Path::new(src);

        if src_path.is_dir() {
            return Err(
                shell::format_error(
                    "cp",
                    ErrorKind::IsADirectory,
                    &format!("{}: Is a directory", src)
                )
            );
        }

        let dest_path = if dest.is_dir() {
            let filename = match src_path.file_name() {
                Some(name) => name,
                None => {
                    return Err(
                        shell::format_error(
                            "cp",
                            ErrorKind::InvalidInput,
                            &format!("{}: Invalid filename", src)
                        )
                    );
                }
            };

            dest.join(filename)
        } else {
            dest.to_path_buf()
        };

        if let Err(e) = fs::copy(src_path, &dest_path) {
            let msg = match e.kind() {
                ErrorKind::NotFound => format!("{}: No such file or directory", src),

                ErrorKind::PermissionDenied => format!("{}: Permission denied", src),

                ErrorKind::IsADirectory => format!("{}: Is a directory", src),

                _ => format!("{}: {}", src, e),
            };

            return Err(shell::format_error("cp", e.kind(), &msg));
        }
    }

    Ok(())
}
