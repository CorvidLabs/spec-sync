# Lesson bundle: other-write-errors-still-panic-and-test-output-stays-captured

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Other write errors still panic and test output stays captured
- **Kind**: BugFix
- **Specs**: cli
- **Paths**: src/main.rs, tests/integration/closed_stdout.rs
- **Acceptance**: A write error other than a closed pipe still panics and is reported as a bug with exit 1 (tested on Linux with /dev/full); under cargo test the printing macros go through libtest's output capture; the CHANGELOG says watch stops at its next output

## Evidence

- Verification commit: `5113eb51b60bb6ce819488697f08c0aed784ba73`
- Base commit: `e853e93a3ef0ab58e0467e2ba5c01a3040b3477f`
- Verified by: `specsync check --spec cli`

## From the change's context.md

# Context

Follow-up to PR #1048, from corvid-agent's review (point 2) and t384's nits: the untested "any
other write error still panics" criterion, libtest capture, and the watch wording.

## From the change's design.md

# Design

The closed-pipe handling in `write_output_stream` now delegates to `std::print!`/`std::eprint!` when
`cfg!(test)`, so unit tests keep libtest's output capture. The integration test points stdout at
`/dev/full` (Linux) and asserts exit 1 with the panic reported as a bug.

## From the change's testing.md

# Testing

Integration `a_write_error_other_than_a_closed_pipe_still_panics` (Linux only; CI runs it). A probe
unit test that prints is captured again under `cargo test` (checked by hand). The three
`closed_stdout` tests still pass.

## Where these lessons go

- `specs/cli/cli.3md`
