use std::io::{ Stdout, Error, Write };
use std::env;
use std::fs;

use crate::parser;
use crate::commands;

const CLEAR_ALL: &str = "\x1b[2J\x1b[3J\x1b[H";

const BLUE: &str = "\x1b[38;2;97;175;239m";
const GREEN: &str = "\x1b[32m";

pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

use libc::{ ECHO, ICANON, TCSANOW, STDIN_FILENO };
use std::mem;

struct RawMode {
    orig: libc::termios,
}

impl RawMode {
    fn new() -> Self {
        unsafe {
            let mut orig: libc::termios = mem::zeroed();
            libc::tcgetattr(STDIN_FILENO, &mut orig);

            let mut raw = orig;
            raw.c_lflag &= !(ICANON | ECHO);

            libc::tcsetattr(STDIN_FILENO, TCSANOW, &raw);
            RawMode { orig }
        }
    }

    fn disable(&self) {
        unsafe {
            libc::tcsetattr(STDIN_FILENO, TCSANOW, &self.orig);
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        self.disable();
    }
}

// ===== Reading =====

fn read_char() -> Option<u8> {
    let mut buf = [0u8; 1];
    let n = unsafe { libc::read(libc::STDIN_FILENO, buf.as_mut_ptr() as *mut libc::c_void, 1) };
    if n == 1 {
        Some(buf[0])
    } else {
        None
    }
}

fn consume_escape_sequence() {
    if let Some(b'[') = read_char() {
        match read_char() {
            Some(b'3') => {
                let _ = read_char();
            }
            _ => {}
        }
    }
}

fn handle_backspace(buf: &mut String, out: &mut Stdout) -> Result<(), Error> {
    if buf.is_empty() {
        return Ok(());
    }
    buf.pop();
    // "\x08": moves the cursor back
    //  "\x1b[K": clears everything after the cursor
    write!(out, "\x08\x1b[K")?;
    out.flush()
}

fn handle_character(ch: u8, buf: &mut String, out: &mut Stdout) -> Result<(), Error> {
    if !(32..127).contains(&ch) {
        return Ok(());
    }
    let c = ch as char;
    buf.push(c);
    write!(out, "{c}")?;
    out.flush()
}

fn handle_enter(out: &mut Stdout) -> Result<(), Error> {
    write!(out, "\r\n")?;
    out.flush()
}

fn handle_tab(buf: &mut String, out: &mut Stdout) -> Result<(), Error> {
    if !buf.contains(" ") {
        return Ok(());
    }

    let last = get_last_word(buf);
    let matches = find_matches(last);

    match matches.len() {
        0 => {}
        1 => {
            let completion = &matches[0];
            let suffix = &completion[last.len()..];
            buf.push_str(suffix);
            write!(out, "{suffix}")?;
            out.flush()?;
        }
        _ => {
            writeln!(out)?;
            for m in &matches {
                writeln!(out, "  {m}")?;
            }
            display_prompt(out)?;
            write!(out, "{buf}")?;
            out.flush()?;
        }
    }
    Ok(())
}

fn get_last_word(buf: &str) -> &str {
    buf.split_whitespace().last().unwrap_or("")
}

fn find_matches(prefix: &str) -> Vec<String> {
    // Parse parts
    let (dir, file) = if prefix.contains("/") {
        let idx = prefix.rfind("/").unwrap();
        let dir = &prefix[..=idx];
        let file = &prefix[idx + 1..];

        (dir, file)
    } else {
        (".", prefix)
    };

    // Get dir's entries
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => {
            return vec![];
        }
    };

    // Collect matches
    let mut matches = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with(&file) {
            let full = if dir == "." { name } else { format!("{}{}", dir, name) };

            if entry.path().is_dir() {
                matches.push(format!("{}/", full));
            } else {
                matches.push(full);
            }
        }
    }

    matches
}

pub fn read(out: &mut Stdout) -> Result<Option<String>, Error> {
    let mut buf = String::new();

    loop {
        let ch = match read_char() {
            Some(c) => c,
            None => {
                return Ok(None);
            }
        };

        match ch {
            b'\n' | b'\r' => {
                handle_enter(out)?;
                break;
            }

            b'\x1b' => {
                consume_escape_sequence();
            }

            127 => {
                handle_backspace(&mut buf, out)?;
            }

            // Ctrl D
            4 => {
                return Ok(None);
            }

            b'\t' => {
                handle_tab(&mut buf, out)?;
            }
            _ => {
                handle_character(ch, &mut buf, out)?;
            }
        }
    }

    Ok(Some(buf))
}

pub fn reset(out: &mut Stdout) -> Result<(), Error> {
    write!(out, "{CLEAR_ALL}")?;
    out.flush()
}

// ===== Main =====
pub fn run(out: &mut Stdout) -> Result<(), Error> {
    let _mode = RawMode::new();

    loop {
        display_prompt(out)?;

        let input = match read(out)? {
            Some(i) => i,
            None => {
                break;
            }
        };

        let (cmd, args) = match parser::parse(&input) {
            Ok(parts) => parts,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        if cmd == "exit" {
            break;
        }

        if let Err(e) = commands::exec(&cmd, args) {
            eprintln!("{}", e);
        }
    }

    Ok(())
}

fn display_prompt(out: &mut Stdout) -> Result<(), Error> {
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

    write!(out, "{BOLD}{BLUE}{p}{RESET} {GREEN}➜{RESET} ")?;
    out.flush()?;

    Ok(())
}
