use crate::error::ShellError;

pub fn run() -> Result<(), ShellError> {
    println!("osh - a minimalist Unix-like shell");
    println!();
    println!("Built-in commands:");
    println!("  echo [args...]          Print arguments to stdout");
    println!("  cd [dir]                Change directory (default: $HOME)");
    println!("  pwd                     Print current directory");
    println!("  ls [-l] [-a] [-F]       List directory contents");
    println!("  cat [files...]          Print file contents");
    println!("  cp [srcs...] dest       Copy files");
    println!("  mv [srcs...] dest       Move files");
    println!("  rm [-r] [paths...]      Remove files or directories");
    println!("  mkdir [dirs...]         Create directories");
    println!("  help                    Show this help message");
    println!("  exit                    Exit the shell");
    Ok(())
}
