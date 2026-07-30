mod shell;
mod parser;
mod commands;

use std::io;
use shell::{ BOLD, RESET };

const RED: &str = "\x1b[38;2;224;108;117m";

fn display_error(msg: &str) {
    eprintln!("{BOLD}{RED}ERROR{RESET}: {msg}")
}

fn main() {
    let mut input = io::stdin();
    let mut output = io::stdout();

    if let Err(e) = shell::reset(&mut output) {
        display_error(&format!("Failed to reset: {}", e));
        return;
    }

    if let Err(e) = shell::run(&mut input, &mut output) {
        display_error(&format!("{}", e));
    }
}
