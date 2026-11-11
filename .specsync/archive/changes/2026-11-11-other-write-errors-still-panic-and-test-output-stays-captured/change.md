---
id: other-write-errors-still-panic-and-test-output-stays-captured
state: archived
type: bug_fix
base_commit: e853e93a3ef0ab58e0467e2ba5c01a3040b3477f
---

# Other write errors still panic and test output stays captured

## Intent

other write errors still panic and test output stays captured

## Affected Canonical Specs

- `cli`

## Acceptance Criteria

- A write error other than a closed pipe still panics and is reported as a bug with exit 1 (tested on Linux with /dev/full); under cargo test the printing macros go through libtest's output capture; the CHANGELOG says watch stops at its next output

## No-spec Rationale

Follow-up to #1048: a test for the existing 'any other write error still panics' behavior, and test-build output through libtest capture; no spec text
