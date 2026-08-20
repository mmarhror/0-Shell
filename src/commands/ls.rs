use std::fs;
use std::os::unix::fs::{ FileTypeExt, PermissionsExt, MetadataExt };
use std::path::Path;
use std::ffi::CStr;
use std::time::SystemTime;
use std::cmp::Ordering;

use libc;
use chrono::{ DateTime, Local };
use xattr;

use crate::error::{ ShellError, CommandError };

use crate::shell::RESET;
const COLOR_DIR: &str = "\x1b[1;34m";
const COLOR_LINK: &str = "\x1b[1;36m";
const COLOR_EXEC: &str = "\x1b[1;32m";
const COLOR_FIFO: &str = "\x1b[33m";
const COLOR_SOCKET: &str = "\x1b[1;35m";

#[derive(Default)]
struct Flags {
    all: bool,
    long: bool,
    class: bool,
}

impl Flags {
    fn new() -> Self {
        Flags::default()
    }

    fn parse(&mut self, args: &Vec<String>) -> Result<Vec<String>, ShellError> {
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
                            self.class = true;
                        }
                        _ => {
                            return Err(
                                ShellError::one("ls", &format!("Invalid option: '-{}'", ch))
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

pub struct Entry {
    pub name: String,
    pub indicator: Option<char>,
    pub color: &'static str,
    long: Option<LongInfo>,
}

impl Entry {
    fn new(path: &Path, name: String, flags: &Flags) -> Result<Self, CommandError> {
        let sym_meta = fs
            ::symlink_metadata(path)
            .map_err(|e| CommandError::new_io("ls", &name, &e))?;

        let ft = sym_meta.file_type();
        let mode = sym_meta.permissions().mode();

        let indicator = if flags.class { get_indicator(&ft, mode) } else { None };

        let color = get_color(&ft, mode);

        let long = if flags.long {
            Some(LongInfo::new(&path, &sym_meta, &ft, flags.class)?)
        } else {
            None
        };

        Ok(Entry {
            name,
            color,
            indicator,
            long,
        })
    }

    fn display_line(&self, ws: &Widths) -> String {
        let long = self.long.as_ref().unwrap();
        let ind = self.indicator.map_or(String::new(), |c| c.to_string());

        let size_str = match &long.size {
            SizeField::Regular(sz) => format!("{:>sw$}", sz, sw = ws.size),
            SizeField::Device { major, minor } => {
                let dev_str = format!(
                    "{:>mj$}, {:>mn$}",
                    major,
                    minor,
                    mj = ws.major,
                    mn = ws.minor
                );
                format!("{:>sw$}", dev_str, sw = ws.size)
            }
        };

        // Only reserve column space for ACL if has_acl is true
        let acl_str = if ws.has_acl { long.acl.unwrap_or(' ').to_string() } else { String::new() };

        format!(
            "{}{} {:>nw$} {:<ow$} {:<gw$} {} {} {}{}{}{}{}",
            long.perms,
            acl_str,
            long.nlink,
            long.owner,
            long.group,
            size_str,
            long.modified,
            self.color,
            self.name,
            RESET,
            ind,
            long.target,
            nw = ws.nlink,
            ow = ws.owner,
            gw = ws.group
        )
    }

    fn sort_key(&self) -> String {
        self.name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    }
}

enum SizeField {
    Regular(u64),
    Device {
        major: u32,
        minor: u32,
    },
}

struct LongInfo {
    blocks: u64,
    perms: String,
    acl: Option<char>,
    nlink: u64,
    owner: String,
    group: String,
    size: SizeField,
    modified: String,
    target: String,
}

impl LongInfo {
    fn new(
        path: &Path,
        meta: &fs::Metadata,
        ft: &fs::FileType,
        class: bool
    ) -> Result<Self, CommandError> {
        let mode = meta.permissions().mode();
        let type_ch = get_type_char(&ft);

        let target = if ft.is_symlink() { get_target(path, class) } else { String::new() };

        Ok(LongInfo {
            blocks: meta.blocks(),
            perms: get_perms(mode, type_ch),
            acl: get_acl(path),
            nlink: meta.nlink(),
            owner: get_owner(meta.uid()),
            group: get_group(meta.gid()),
            size: get_size(meta, ft),
            modified: get_time(
                meta
                    .modified()
                    .map_err(|e| CommandError::new_io("ls", &path.to_string_lossy(), &e))?
            ),
            target,
        })
    }
}

// ===== Fetch =====
fn get_type_char(ft: &fs::FileType) -> char {
    if ft.is_symlink() {
        'l'
    } else if ft.is_dir() {
        'd'
    } else if ft.is_fifo() {
        'p'
    } else if ft.is_socket() {
        's'
    } else if ft.is_block_device() {
        'b'
    } else if ft.is_char_device() {
        'c'
    } else {
        '-'
    }
}

fn get_perms(mode: u32, type_ch: char) -> String {
    let mut perms = String::new();

    let is_setuid = (mode & 0o4000) != 0;
    let is_setgid = (mode & 0o2000) != 0;
    let is_sticky = (mode & 0o1000) != 0;

    perms.push(type_ch);

    perms.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    perms.push(match (is_setuid, (mode & 0o100) != 0) {
        (true, true) => 's',
        (true, false) => 'S',
        (false, true) => 'x',
        (false, false) => '-',
    });

    perms.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    perms.push(match (is_setgid, (mode & 0o010) != 0) {
        (true, true) => 's',
        (true, false) => 'S',
        (false, true) => 'x',
        (false, false) => '-',
    });

    perms.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    perms.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    perms.push(match (is_sticky, (mode & 0o001) != 0) {
        (true, true) => 't',
        (true, false) => 'T',
        (false, true) => 'x',
        (false, false) => '-',
    });

    perms
}

fn get_acl(path: &Path) -> Option<char> {
    match xattr::get(path, "system.posix_acl_access") {
        Ok(Some(data)) if data.len() > 28 => Some('+'),
        _ => None,
    }
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

fn get_group(gid: u32) -> String {
    unsafe {
        let gr = libc::getgrgid(gid);
        if gr.is_null() {
            return gid.to_string();
        }

        CStr::from_ptr((*gr).gr_name).to_string_lossy().to_string()
    }
}

fn get_size(meta: &fs::Metadata, ft: &fs::FileType) -> SizeField {
    if ft.is_block_device() || ft.is_char_device() {
        let rdev = meta.rdev() as libc::dev_t;
        let major = libc::major(rdev) as u32;
        let minor = libc::minor(rdev) as u32;
        SizeField::Device { major, minor }
    } else {
        SizeField::Regular(meta.len())
    }
}

fn get_time(modified: SystemTime) -> String {
    let now = SystemTime::now();
    let six_months_secs = 182 * 24 * 60 * 60;

    let date: DateTime<Local> = modified.into();

    let is_old_or_future = match now.duration_since(modified) {
        Ok(dur) => dur.as_secs() > six_months_secs,
        Err(_) => true,
    };

    if is_old_or_future {
        date.format("%b %e  %Y").to_string()
    } else {
        date.format("%b %e %H:%M").to_string()
    }
}

fn get_color(ft: &fs::FileType, mode: u32) -> &'static str {
    if ft.is_symlink() {
        COLOR_LINK
    } else if ft.is_dir() {
        COLOR_DIR
    } else if ft.is_fifo() {
        COLOR_FIFO
    } else if ft.is_socket() {
        COLOR_SOCKET
    } else if (mode & 0o111) != 0 {
        COLOR_EXEC
    } else {
        ""
    }
}

fn get_indicator(ft: &fs::FileType, mode: u32) -> Option<char> {
    if ft.is_dir() {
        Some('/')
    } else if ft.is_fifo() {
        Some('|')
    } else if ft.is_socket() {
        Some('=')
    } else if ft.is_file() && (mode & 0o111) != 0 {
        Some('*')
    } else {
        None
    }
}

fn get_target(path: &Path, classify: bool) -> String {
    let link = match fs::read_link(path) {
        Ok(p) => p,
        Err(_) => {
            return String::new();
        }
    };

    let ind = if classify {
        let resolved = if link.is_absolute() {
            link.clone()
        } else {
            path.parent().unwrap_or(Path::new(".")).join(&link)
        };

        fs::metadata(&resolved)
            .ok()
            .and_then(|m| {
                let ft = m.file_type();
                let mode = m.permissions().mode();
                get_indicator(&ft, mode)
            })
            .map(|c| c.to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    format!(" -> {}{}", link.to_string_lossy(), ind)
}

// ===== Format =====
// Short
fn print_short(ents: &[Entry]) {
    if ents.is_empty() {
        return;
    }

    let col_w = get_col_width(ents) + 2;
    let width = get_terminal_width();

    let cols = (width / col_w).max(1);
    let rows = ((ents.len() as f64) / (cols as f64)).ceil() as usize;

    for row in 0..rows {
        for col in 0..cols {
            let i = row + col * rows;

            if i >= ents.len() {
                continue;
            }

            let ind = ents[i].indicator.map_or(String::new(), |c| c.to_string());
            let display_len =
                ents[i].name.len() + (if ents[i].indicator.is_some() { 1 } else { 0 });

            let padding = col_w - display_len;

            if col == cols - 1 {
                print!("{}{}{}{}", ents[i].color, ents[i].name, RESET, ind);
            } else {
                print!("{}{}{}{}{}", ents[i].color, ents[i].name, RESET, ind, " ".repeat(padding));
            }
        }
        println!();
    }
}

fn get_terminal_width() -> usize {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            ws.ws_col as usize
        } else {
            80
        }
    }
}

fn get_col_width(ents: &[Entry]) -> usize {
    let mut max = 0;
    for ent in ents {
        let w = ent.name.len() + (if ent.indicator.is_some() { 1 } else { 0 });
        max = w.max(max);
    }

    max
}

// Long

fn print_long(ents: &[Entry]) {
    let widths = get_widths(ents);

    for ent in ents {
        println!("{}", ent.display_line(&widths));
    }
}

struct Widths {
    nlink: usize,
    owner: usize,
    group: usize,
    size: usize,
    major: usize,
    minor: usize,
    has_acl: bool,
}

fn get_widths(ents: &[Entry]) -> Widths {
    let mut w = Widths {
        nlink: 0,
        owner: 0,
        group: 0,
        size: 0,
        major: 0,
        minor: 0,
        has_acl: false,
    };

    let mut has_device = false;

    for ent in ents {
        if let Some(ref long) = ent.long {
            w.nlink = w.nlink.max(long.nlink.to_string().len());
            w.owner = w.owner.max(long.owner.len());
            w.group = w.group.max(long.group.len());

            if long.acl.is_some() {
                w.has_acl = true;
            }

            match &long.size {
                SizeField::Regular(sz) => {
                    w.size = w.size.max(sz.to_string().len());
                }
                SizeField::Device { major, minor } => {
                    has_device = true;
                    w.major = w.major.max(major.to_string().len());
                    w.minor = w.minor.max(minor.to_string().len());
                }
            }
        }
    }

    if has_device {
        let dev_column_width = w.major + 2 + w.minor;
        w.size = w.size.max(dev_column_width);
    }

    w
}

// ===== Run =====

fn print_entries(ents: &[Entry], flags: &Flags, show_total: bool) {
    if ents.is_empty() {
        if flags.long && show_total {
            println!("total 0");
        }
        return;
    }

    if flags.long {
        if show_total {
            let blocks: u64 = ents
                .iter()
                .map(|e| e.long.as_ref().unwrap().blocks)
                .sum();
            println!("total {}", blocks / 2);
        }
        print_long(ents);
    } else {
        print_short(ents);
    }
}

fn print_dirs(name: &str, dirs: &[Entry], flags: &Flags, show_header: bool) {
    if show_header {
        println!("{}:", name);
    }
    print_entries(dirs, flags, true);
}

fn print_files(files: &[Entry], flags: &Flags) {
    print_entries(files, flags, false);
}

fn case_tie_key(s: &str) -> Vec<(char, u8)> {
    s.chars()
        .map(|c| (
            if c.is_uppercase() { c.to_ascii_lowercase() } else { c },
            c.is_uppercase() as u8,
        ))
        .collect()
}

fn compare_entries(a: &Entry, b: &Entry) -> Ordering {
    a.sort_key()
        .cmp(&b.sort_key())
        .then_with(|| case_tie_key(&a.name).cmp(&case_tie_key(&b.name)))
        .then_with(|| a.name.cmp(&b.name))
}

pub fn run(args: Vec<String>) -> Result<(), ShellError> {
    // Init & Parse flags
    let mut flags = Flags::new();
    let paths = flags.parse(&args)?;

    let targets = if paths.is_empty() { vec![String::from(".")] } else { paths };

    // Collect Entries & Errors
    let mut files: Vec<Entry> = Vec::new();
    let mut dirs: Vec<(String, Vec<Entry>)> = Vec::new();
    let mut errors: Vec<CommandError> = Vec::new();

    for target in targets {
        let path = Path::new(&target);

        let meta = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) => {
                errors.push(CommandError::new_io("ls", &target, &e));
                continue;
            }
        };

        if meta.file_type().is_dir() {
            match collect_dir(path, target.clone(), &flags) {
                Ok(mut ents) => {
                    ents.sort_by(compare_entries);
                    dirs.push((target, ents));
                }
                Err(e) => errors.push(e),
            }
        } else {
            match collect_file(path, target.clone(), &flags) {
                Ok(ent) => files.push(ent),
                Err(e) => errors.push(e),
            }
        }
    }

    // Print Errors
    for err in &errors {
        eprintln!("{}", err);
    }

    // List
    let show_header = dirs.len() > 1 || (!files.is_empty() && !dirs.is_empty());
    let mut printed = false;

    if !files.is_empty() {
        files.sort_by(compare_entries);
        print_files(&files, &flags);
        printed = true;
    }

    for (name, dir) in dirs {
        if printed {
            println!();
        }

        print_dirs(&name, &dir, &flags, show_header);
        printed = true;
    }

    Ok(())
}

fn collect_file(path: &Path, name: String, flags: &Flags) -> Result<Entry, CommandError> {
    Entry::new(path, name.clone(), flags)
}

fn collect_dir(path: &Path, name: String, flags: &Flags) -> Result<Vec<Entry>, CommandError> {
    let mut ents: Vec<Entry> = Vec::new();

    if flags.all {
        ents.push(collect_file(path.join(".").as_path(), ".".to_string(), flags)?);
        ents.push(collect_file(path.join("..").as_path(), "..".to_string(), flags)?);
    }

    let entries = fs::read_dir(path).map_err(|e| { CommandError::new_io("ls", &name, &e) })?;

    for dir_entry in entries {
        let dir_entry = dir_entry.map_err(|e| CommandError::new_io("ls", &name, &e))?;

        let name = dir_entry.file_name().to_string_lossy().to_string();

        if !flags.all && name.starts_with('.') {
            continue;
        }

        ents.push(collect_file(&dir_entry.path(), name, flags)?);
    }

    Ok(ents)
}
