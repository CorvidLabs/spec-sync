---
change: a-closed-output-never-crashes-specsync-or-hides-its-exit-status
artifact: design
---

# Design

Rust ignores SIGPIPE, so writing to a pipe whose reader is gone fails with `BrokenPipe`, and
`println!` and `eprintln!` panic on it. That made `specsync … | head` end with exit 101 instead
of the command's status.

`src/main.rs` defines `println!`, `print!`, `eprintln!` and `eprint!` before its `mod` items,
which shadows the prelude macros crate-wide. They call `write_output_stream`. On `BrokenPipe` it
marks the stream closed and drops further output to it, so the command runs on to its own exit
status. Any other write error still panics, as before. `watch` only reports, so it returns once its
stdout is closed instead of watching on unseen.

Restoring SIGPIPE's default was rejected: the process would die by signal (status 141), which still
hides the verdict the issue asks to keep.
