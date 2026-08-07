use std::fmt;
use std::io;

pub struct CommandError {
    cmd: String,
    msg: String,
}

impl CommandError {
    pub fn new(cmd: &str, msg: &str) -> Self {
        CommandError {
            cmd: cmd.to_string(),
            msg: msg.to_string(),
        }
    }

    pub fn new_io(cmd: &str, path: &str, e: &io::Error) -> Self {
        let msg = match e.kind() {
            io::ErrorKind::NotFound => format!("{}: No such file or directory", path),
            io::ErrorKind::PermissionDenied => format!("{}: Permission denied", path),
            io::ErrorKind::AlreadyExists => format!("{}: File exists", path),
            io::ErrorKind::IsADirectory => format!("{}: Is a directory", path),
            _ => format!("{}: {}", path, e),
        };
        CommandError::new(cmd, &msg)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.cmd.is_empty() {
            write!(f, "{}", self.msg)
        } else {
            write!(f, "{}: {}", self.cmd, self.msg)
        }
    }
}

pub enum ShellError {
    One(CommandError),
    Many(Vec<CommandError>),
}

impl ShellError {
    pub fn one(cmd: &str, msg: &str) -> Self {
        ShellError::One(CommandError::new(cmd, msg))
    }

    pub fn one_io(cmd: &str, path: &str, e: &io::Error) -> Self {
        ShellError::One(CommandError::new_io(cmd, path, e))
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::One(e) => write!(f, "{}", e),
            ShellError::Many(errs) => {
                for e in errs {
                    writeln!(f, "{}", e)?;
                }
                Ok(())
            }
        }
    }
}
