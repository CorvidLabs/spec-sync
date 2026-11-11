# Lesson bundle: a-closed-output-never-crashes-specsync-or-hides-its-exit-status

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A closed output never crashes specsync or hides its exit status
- **Kind**: BugFix
- **Specs**: cli, watch
- **Paths**: src/main.rs, src/watch.rs, tests/integration.rs, tests/integration/closed_stdout.rs
- **Acceptance**: With its stdout (or stderr) reader gone, specsync drops further output to that stream and exits with the command's own status, 0 or 1, never a panic or exit 101; watch stops once nobody reads its output; any other write error still panics

## Evidence

- Verification commit: `83eb01103c77e20d2e92c9b16196d84bb99b7fc8`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec cli --spec watch`

## From the change's context.md

# Context

Issue #1036 was filed from the spec-sync worker's report: `specsync change status <id> | head -1`
panicked with exit 101 on 6.2.0 and main, masking the real 0/1 verdict. REQ-cli already requires
exit 0 on success and 1 on errors, so this is a bug fix with no requirement text.

## From the change's design.md

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

## From the change's testing.md

# Testing

`tests/integration/closed_stdout.rs` starts SpecSync with its stdout reader already dropped:

- a passing `check` (text and `--json`) exits 0;
- a failing `check` exits 1, with stderr closed too;
- `watch` exits 0 on its own.

None prints panic text. All three fail on main (panic, exit 1 instead of 0, and watch running on).

## Where these lessons go

- `specs/cli/cli.3md`
- `specs/watch/watch.3md`
