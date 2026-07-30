use std::io::{ Error, ErrorKind };
use std::fs;
use std::os::unix::fs::{ MetadataExt, FileTypeExt, PermissionsExt };
use std::path::Path;
use std::ffi::CStr;
use std::time::SystemTime;
use std::mem;

use libc;
use chrono::{ DateTime, Local };

use crate::shell;
#[derive(Default)]
struct Flags {
    all: bool,
    long: bool,
    flags: bool,
}

impl Flags {
    fn new() -> Self {
        Flags::default()
    }

    fn parse(&mut self, args: &Vec<String>) -> Result<Vec<String>, Error> {
        let mut paths: Vec<String> = Vec::new();

        for arg in args {
            if arg.starts_with("-") {
                for ch in arg[1..].chars() {
                    match ch {
                        'a' => {
                            self.all = true;
                        }
                        'l' => {
                            self.long = true;
                        }
                        'F' => {
                            self.flags = true;
                        }
                        _ => {
                            return Err(
                                shell::format_error(
                                    "ls",
                                    ErrorKind::InvalidInput,
                                    &format!("invalid option: '-{}'", arg)
                                )
                            );
                        }
                    }
                }
            } else {
                paths.push(arg.clone());
            }
        }

        Ok(paths)
    }
}

#[derive(Debug)]
struct Entry {
    perms: Option<String>,
    nlink: Option<u64>,
    owner: Option<String>,
    group: Option<String>,
    size: Option<u64>,
    modified: Option<String>,
    name: String,
    indicator: Option<char>,
}

impl Entry {
    fn new(path: &Path, flags: &Flags) -> Result<Self, Error> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let mut ent = Entry {
            perms: None,
            nlink: None,
            owner: None,
            group: None,
            size: None,
            modified: None,
            name,
            indicator: None,
        };

        let metadata = fs::metadata(path)?;

        if flags.long {
            let is_dir = metadata.is_dir();

            ent.perms = Some(get_perms(metadata.permissions().mode(), is_dir));
            ent.nlink = Some(metadata.nlink());
            ent.owner = Some(get_owner(metadata.uid()));
            ent.group = Some(get_group(metadata.gid()));
            ent.size = Some(metadata.len());
            ent.modified = Some(get_time(metadata.modified()?));
        }

        if flags.flags {
            ent.indicator = get_indicator(path)?;
        }

        Ok(ent)
    }

    fn line(&self, flags: &Flags) {}
}

// ===== Fetch =====
fn get_perms(mode: u32, is_dir: bool) -> String {
    let mut perms = String::new();

    perms.push(if is_dir { 'd' } else { '-' });

    perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o100) != 0 { 'x' } else { '-' });

    perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o010) != 0 { 'x' } else { '-' });

    perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    perms.push(if (mode & 0o001) != 0 { 'x' } else { '-' });

    perms
}

fn get_owner(uid: u32) -> String {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return uid.to_string();
        }

        CStr::from_ptr((*pw).pw_name).to_string_lossy().to_string()
    }
}

fn get_group(uid: u32) -> String {
    unsafe {
        let gr = libc::getgrgid(uid);
        if gr.is_null() {
            return uid.to_string();
        }

        CStr::from_ptr((*gr).gr_name).to_string_lossy().to_string()
    }
}

fn get_time(modified: SystemTime) -> String {
    let date: DateTime<Local> = modified.into();

    date.format("%d %e %H:%M").to_string()
}

fn get_indicator(path: &Path) -> Result<Option<char>, Error> {
    let metadata = fs::symlink_metadata(path)?;

    let file_type = metadata.file_type();

    if file_type.is_symlink() {
        return Ok(Some('@'));
    }

    if file_type.is_dir() {
        return Ok(Some('/'));
    }

    if file_type.is_fifo() {
        return Ok(Some('|'));
    }

    if file_type.is_socket() {
        return Ok(Some('='));
    }

    if file_type.is_file() {
        let mode = metadata.permissions().mode();
        if (mode & 0o111) != 0 {
            return Ok(Some('*'));
        }
    }

    Ok(None)
}

// ===== Format =====

fn format_short(ents: &[Entry]) {
    let col_w = get_col_width(ents);

    let mut line = String::new();
    for ent in ents {
        
    }
}

fn get_col_width(ents: &[Entry]) -> usize {
    let mut max = 0;
    for ent in ents {
        let w = ent.name.len() + (if ent.indicator.is_some() { 1 } else { 0 });
        if w > max {
            max = ent.name.len();
        }
    }
    max
}

fn get_terminal_width() -> usize {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        libc::ioctl(1, libc::TIOCGWINSZ, &mut ws);
        ws.ws_col as usize
    }
}

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let mut flags = Flags::new();
    let paths = flags.parse(&args)?;

    let targets = if paths.is_empty() { vec![String::from(".")] } else { paths };

    let mut ents: Vec<Entry> = Vec::new();

    for target in targets {
        let path = Path::new(&target);

        if path.is_dir() {
            for file in fs::read_dir(path)? {
                ents.push(Entry::new(&file?.path(), &flags)?);
            }
        } else {
            ents.push(Entry::new(&path, &flags)?);
        }
    }

    ents.sort_by(|a, b| a.name.cmp(&b.name));



    Ok(())
}
