# 0-Shell

A minimalist Unix-like shell written in Rust.

## Features

- Raw-mode line editor with arrow-key handling, backspace, and Ctrl+D
- Tab completion for files and directories
- Colored prompt showing the current directory
- Quote-aware command parsing
- Built-in commands: `echo`, `cd`, `pwd`, `ls`, `cat`, `cp`, `mv`, `rm`, `mkdir`, `help`, `exit`

## Commands

> **Note:** Run `help` inside the shell to see the available commands.

| Command | Description |
| --- | --- |
| `echo [args...]` | Print arguments to stdout |
| `cd [dir]` | Change directory (default: `$HOME`) |
| `pwd` | Print current directory |
| `ls [-l] [-a] [-F]` | List directory contents |
| `cat [files...]` | Print file contents |
| `cp [srcs...] dest` | Copy files |
| `mv [srcs...] dest` | Move files |
| `rm [-r] [paths...]` | Remove files or directories |
| `mkdir [dirs...]` | Create directories |
| `help` | Show help message |
| `exit` | Exit the shell |

## Project Tree

```
0-Shell/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
└── src/
    ├── main.rs            Entry point
    ├── shell.rs           REPL loop, line editor, tab completion, prompt
    ├── parser.rs          Quote-aware command parser
    ├── error.rs           Shell error types
    └── commands/
        ├── mod.rs         Command dispatcher
        ├── help.rs
        ├── echo.rs
        ├── pwd.rs
        ├── cd.rs
        ├── mkdir.rs
        ├── cat.rs
        ├── cp.rs
        ├── mv.rs
        ├── rm.rs
        └── ls.rs
```

## Build & Run

```sh
cargo run
```

Requires Rust (edition 2024). Unix-only (uses `libc`/`termios`).
