---
change: CHG-0115-a-fix-request-that-could-not-be-applied-must-be-reported-not-reported-as-succes
artifact: testing
---

# Testing

## Strategy

Making a command fail is easy; making it fail **only** where it should is the work. Three
fixtures bracket it, and the third is the one that keeps this from being an over-correction.

## Verified by hand

| fixture | before | after |
|---|---|---|
| read-only spec, `--fix` | exit 0, silent | `error: specs/alpha/alpha.spec.md: could not be written, so the fix was not applied: Permission denied (os error 13)`, exit 1 |
| **fresh writable spec**, `--fix` | `added 1 export(s)` | unchanged — `added 1 export(s)`, exit 0, and the export is present in the file afterwards |
| **read-only spec, `--fix --dry-run`** | exit 0 | **exit 0**, `would add 1 export(s)` |

The third row is load-bearing. A dry run attempts no write, so it fails at nothing — which
proves the new failure is tied to an **attempted write** rather than to the file's mode.
Without it, this change could not be distinguished from one that refuses any read-only tree.

The second row was initially inconclusive: the writable fixture had already been repaired by
an earlier run, so there was nothing left to add and its exit 0 proved nothing. It was rebuilt
from scratch, and the assertion now also checks the export is really in the file.

## Regression surface

2210 unit and 331 integration tests pass unchanged. The change adds a failure path where
there was none and leaves the success path untouched.

## Not covered

No unit test asserts the new message. `auto_fix_specs` has no focused harness in this
change's scope; behavioural pinning belongs in the sandbox, where drill 041 already drives
`--fix`-adjacent flows over generated specs.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-check-008 | `cargo test` (2210 + 331, 0 failures) plus the three fixtures above: an unwritable spec now names the path and the OS error and exits 1, a fresh writable spec is still repaired with the export verified present afterwards, and a dry run against the same unwritable spec still exits 0 — which confines the new failure to an attempted write |
