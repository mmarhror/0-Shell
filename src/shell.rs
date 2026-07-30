use std::io::{ Stdin, Stdout, Error, ErrorKind, Write };
use std::env;

use crate::parser;
use crate::commands;

const CLEAR_ALL: &str = "\x1b[2J\x1b[3J\x1b[H";

const WHITE: &str = "\x1b[38;2;171;178;191m";
const BLUE: &str = "\x1b[38;2;97;175;239m";
const GREEN: &str = "\x1b[32m";

pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

pub fn format_error(cmd: &str, kind: ErrorKind, msg: &str) -> Error {
    Error::new(kind, format!("{}: {}", cmd, msg))
}

pub fn display_error(msg: &str) {
    eprintln!("{BOLD}{WHITE}osh{RESET} {BOLD}{GREEN}➜{RESET} {msg}");
}

pub fn reset(out: &mut Stdout) -> Result<(), Error> {
    print!("{CLEAR_ALL}");
    out.flush()
}

pub fn run(inp: &mut Stdin, out: &mut Stdout) -> Result<(), Error> {
    let mut buf = String::new();

    loop {
        display_prompt(out)?;
        let bytes = read(&inp, &mut buf)?;

        if bytes == 0 {
            break;
        }

        let input = buf.trim();
        if input.is_empty() {
            continue;
        }

        let (cmd, args) = match parser::parse(&input) {
            Ok(parts) => parts,
            Err(e) => {
                display_error(&e.to_string());
                continue;
            }
        };

        if cmd == "exit" {
            break;
        }

        if let Err(e) = commands::exec(&cmd, args) {
            display_error(&e.to_string());
        }
    }

    Ok(())
}

fn display_prompt(output: &mut Stdout) -> Result<(), Error> {
    let p = match env::current_dir() {
        Ok(path) => {
            match env::var("HOME") {
                Ok(home) if path.to_string_lossy() == home => { "~".to_string() }
                _ => {
                    let dir = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or("/".to_string());

                    format!("~/{}", dir)
                }
            }
        }
        Err(_) => "?".to_string(),
    };

    print!("{BOLD}{BLUE}{}{RESET} {GREEN}➜{RESET} ", p);
    output.flush()?;

    Ok(())
}

fn read(input: &Stdin, buf: &mut String) -> Result<usize, Error> {
    buf.clear();

    input.read_line(buf)
}
