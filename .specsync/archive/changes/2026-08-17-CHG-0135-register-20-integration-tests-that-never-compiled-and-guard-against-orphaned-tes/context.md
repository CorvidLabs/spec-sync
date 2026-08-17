---
change: CHG-0135-register-20-integration-tests-that-never-compiled-and-guard-against-orphaned-tes
artifact: context
---

# Context

`tests/integration/regression_w1.rs` arrived in `9a00223b` (#483) carrying 20 tests and
511 lines, and was never `#[path]`-declared in `tests/integration.rs`.

`Cargo.toml` defines no `[[test]]` targets, so Cargo auto-discovers `tests/*.rs`.
`tests/integration/` is a *subdirectory*, so its files are not auto-discovered — each one
must be registered explicitly. Twelve were. One was not.

At the commit this change starts from:

    .rs files in tests/integration/          13
    #[path] declarations in integration.rs   12
    grep -c regression_w1 integration.rs      0

The set difference is exactly one orphan.

## Why it stayed invisible

An unregistered file is never in a compilation unit, so it produces no failures, and a
suite with no failures reads as green. Twenty regression assertions were absent from every
passing CI run they appeared to be part of.

This is the defect class 6.0 is closing, applied to the test harness rather than the
product: **a category is empty for want of input, and the result is read as want of
problems.** Filed as #585.

## What was ruled out

The issue predicted compile-rot and advised expecting the file not to build. It built on
the first attempt, clean, with a single unused import (`predicates::prelude::*`, which the
file never uses). The helper API it depends on — `specsync()`, `valid_spec`,
`write_config`, `setup_minimal_project` — is unchanged.

So this was never drift. It was a registration oversight, and the 20 assertions are being
evaluated against exactly the code they were written for. No assertion needed rewriting to
compile.

## What running them exposed

18 passed, 2 failed. Both failures were real and are filed as product defects, deliberately
**not** fixed here:

- **#605** — `report --require-coverage` is unreachable when staleness is unmeasurable. In a
  non-git tree `report` exits 1 unconditionally, so `--require-coverage 0` also exits 1. No
  coverage threshold can fail at zero, so the gate is not failing — it is not running. Exit
  code and printed percentage disagree.
- **#606** — `deps` emits two `✗` findings for one missing dependency
  (`src/validator.rs:2570` and `src/deps.rs:261`), so one defect is double-counted.

And a third, found by inspection rather than by failure:

- **#607** — the *passing* sibling `report_require_coverage_above_actual_exits_1` would pass
  with `--require-coverage` deleted from the codebase. Same un-gitted fixture, asserts only
  `.failure()`. A green test carrying zero information — the same defect class, inside the
  file being resurrected.

## Constraint

No product code may change in this change. #605 and #606 stay open. The fixtures here stop
the tests being *blind* to those defects; they do not fix or hide them.
