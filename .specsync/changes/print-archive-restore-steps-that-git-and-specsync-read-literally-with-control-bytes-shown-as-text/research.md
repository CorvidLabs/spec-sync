---
change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
artifact: research
---

# Research

Checked by hand on 2026-11-13 (git 2.50.1, GNU bash 3.2.57, macOS), in a scratch repository:

- `git diff --quiet HEAD -- ':(literal)pkg/fixA'` exits 1 for an edited `fixA`, and a
  `:(literal)pkg/fix*` probe does not see that edit.
- `git restore --source=HEAD --staged --worktree -- ':(literal)pkg/fix*'` restored `pkg/fix*` and
  left the edit to `pkg/fixA` in place. The plain-path form restores both (#929's repro).
- `bash -c "git restore ... -- $':(literal)pkg/a\\nb\\x1b[31mc'"` restored the directory whose name
  holds a newline and an ESC byte. `\\xHH` takes at most two hex digits, so a following character
  is never absorbed into the escape.
- `$'…'` is in bash, zsh and ksh, and in POSIX.1-2024 `sh`. Older `dash` lacks it. Only a name
  that holds a control byte is printed this way; every other path keeps single quotes.
- Clap reads `specsync change status -- --json` as `Status { id: Some("--json") }` (the carried
  test parses the printed command with `Cli::try_parse_from`).
