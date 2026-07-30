use std::io::{ Error, ErrorKind };
use std::fs;
use std::path::Path;

use crate::shell;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() {
        return Err(shell::format_error("mv", ErrorKind::InvalidInput, "Missing operand"));
    }

    if args.len() < 2 {
        return Err(shell::format_error("mv", ErrorKind::InvalidInput, "Missing destination"));
    }

    let dest = Path::new(args.last().unwrap());
    let srcs = &args[..args.len() - 1];

    if srcs.len() > 1 && !dest.is_dir() {
        return Err(
            shell::format_error(
                "mv",
                ErrorKind::InvalidInput,
                &format!("{}: Not a directory", dest.display())
            )
        );
    }

    for src in srcs {
        let src_path = Path::new(src);

        let dest_path = if dest.is_dir() {
            let filename = match src_path.file_name() {
                Some(name) => name,
                None => {
                    eprintln!(
                        "{}",
                        shell::format_error(
                            "mv",
                            ErrorKind::InvalidInput,
                            &format!("{}: Invalid filename", src)
                        )
                    );
                    continue;
                }
            };

            dest.join(filename)
        } else {
            dest.to_path_buf()
        };

        if let Err(e) = fs::rename(src_path, dest_path) {
            let msg = match e.kind() {
                ErrorKind::NotFound => format!("{}: No such file or directory", src),

                ErrorKind::PermissionDenied => format!("{}: Permission denied", src),

                ErrorKind::AlreadyExists => format!("{}: File exists", src),

                _ => format!("{}: {}", src, e),
            };

            eprintln!("{}", shell::format_error("mv", e.kind(), &msg));
            continue;
        }
    }

    Ok(())
}
