---
spec: commands.spec.md
---

## Key Decisions

- This is the shared infrastructure layer: `mod.rs` holds the boilerplate every subcommand reuses (config load, spec discovery, filtering, validation pipeline, exit-code logic, GitHub drift issues) and re-exports each `commands::*` submodule.
- Discovery skips `_`-prefixed spec files so template/internal specs never get validated.
- Exit-code logic is split in two: `compute_exit_code` is pure (returns `i32`, easy to unit-test), while `exit_with_status` is the side-effecting twin that prints and calls `process::exit`.
- `run_validation` owns the full text rendering of check output AND the `collect` path that gathers error/warning strings for JSON/markdown/GitHub formats — both share one validation loop and one ignore-rule filter.
- `filter_by_status` reads only the frontmatter section (up to the closing `---`) to avoid re-reading full file bodies that callers parse again later.

## Files to Read First

- `src/commands/mod.rs` — the module itself (all shared functions + submodule re-exports)
- `src/main.rs` — dispatches every `Command` variant into these submodules; also home to the `compute_exit_code` unit tests
- `src/validator.rs` — `find_spec_files`, `validate_spec` (the core called by `run_validation`)
- `src/config.rs` — `load_config` used by `load_and_discover`

## Current Status

Fully implemented and stable. `compute_exit_code` is unit-tested in `src/main.rs`; the surrounding flow is exercised by `tests/integration.rs`. `mod.rs` itself has no `#[cfg(test)]` module.

## Notes

- This module orchestrates library modules rather than containing domain logic.
- `filter_by_status` warns on unrecognized status strings so typos don't silently filter nothing.
