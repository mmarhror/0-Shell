# FLAWS

Scan results (critical only). Line numbers refer to the current source. A suggested fix is listed below each flaw.

1. **`cp file file` destroys data** — `src/commands/cp.rs:61`
   Copying a file onto itself is not guarded. `fs::copy` truncates the destination
   first, so `cp a.txt a.txt` silently empties `a.txt` (verified). GNU cp refuses:
   "cp: 'a' and 'a' are the same file".

   **Fix:** Before copying, compare the source and destination for equality and
   error out if they resolve to the same file. `fs::canonicalize` both sides; if
   the source has no `file_name` (dest is a directory), use `dest.join(filename)`
   first, then reject when the canonical paths match:
   ```rust
   if canonical(src) == canonical(dest_path) {
       return Err(shell::format_error("cp", ErrorKind::InvalidInput,
           &format!("'{}' and '{}' are the same file", src, dest_path.display())));
   }
   ```
   Fall back to comparing `dev`/`ino` via `std::os::unix::fs::MetadataExt` if
   `canonicalize` fails on a missing destination.

2. **`mv` cannot move across filesystems** — `src/commands/mv.rs:53`
   Uses `fs::rename`, which fails with `EXDEV` ("Invalid cross-device link") when
   source and destination are on different filesystems. Real `mv` falls back to
   copy + delete.

   **Fix:** On `ErrorKind::CrossesDevices`, fall back to copy-then-delete:
   ```rust
   if let Err(e) = fs::rename(src_path, &dest_path) {
       if e.kind() == ErrorKind::CrossesDevices {
           fs::copy(src_path, &dest_path)?;
           fs::remove_file(src_path)?;
       } else {
           return Err(shell::format_error("mv", e.kind(), &msg));
       }
   }
   ```
   For directories, use a recursive copy of the tree, or simply propagate the
   error if recursive cross-device moves are out of scope.

3. **`rm` treats unknown options as file names** — `src/commands/rm.rs:24-25`
   `rm -f` attempts to remove a file literally named `-f` (verified) instead of
   recognizing `-f` as a flag. Only the exact first arg `-r` is special-cased;
   combined forms like `-rf` are also treated as paths.

   **Fix:** Parse flags before collecting paths, like `ls::Flags`, instead of
   inspecting only `args[0]`:
   ```rust
   let mut recursive = false;
   let mut force = false;
   let mut paths = Vec::new();
   for arg in &args {
       if arg.starts_with('-') {
           for ch in arg[1..].chars() {
               match ch { 'r' | 'R' => recursive = true, 'f' => force = true, _ => return Err(...) }
           }
       } else {
           paths.push(arg.clone());
       }
   }
   if paths.is_empty() { return Err(shell::format_error("rm", ErrorKind::InvalidInput, "Missing operand")); }
   ```
   Treat `-f` as "ignore nonexistent files" and never pass it to `remove_file`.

4. **Prompt is wrong/meaningless outside `$HOME`** — `src/shell.rs:65-87`
   The prompt always renders `~/<basename>` regardless of actual location. After
   `cd /` it prints `~//` (verified); anywhere else (e.g. `/tmp`) it prints
   `~/tmp`, which is misleading and loses the path context. GNU-style shells only
   use `~` when the cwd is inside `$HOME`.

   **Fix:** Show the real path, replacing only a `$HOME`-prefixed cwd with `~`:
   ```rust
   match env::var("HOME") {
       Ok(home) if cwd.starts_with(&home) => {
           let rest = &cwd[home.len()..];
           format!("~{}", if rest.is_empty() { "" } else { rest })
       }
       _ => cwd,
   }
   ```
   This yields `~`, `~/sub`, and plain `/`, `/tmp/...` for anything outside
   `$HOME`, avoiding the `~//` case. (Optionally show only the last 1–2 path
   components instead of the full path if brevity is desired.)
