---
spec: commands.spec.md
---

## Key Decisions

- This is the shared infrastructure layer: `mod.rs` holds the boilerplate every subcommand reuses (config load, spec discovery, filtering, validation pipeline, exit-code logic, GitHub drift issues) and re-exports each `commands::*` submodule.
- Discovery skips `_`-prefixed spec files so template/internal specs never get validated.
- Exit-code logic is split in two: `compute_exit_code` is pure (returns `i32`, easy to unit-test), while `exit_with_status` is the side-effecting twin that prints and calls `process::exit`.
- `run_validation` owns the full text rendering of check output AND the `collect` path that gathers error/warning strings for JSON/markdown/GitHub formats — both share one validation loop and one ignore-rule filter.
- `run_validation_with_cache` is a private extension of that same loop. It records already-filtered diagnostics into `HashCache` without changing public callers or duplicating validator behavior.
- Global validation input discovery is centralized here because both `check` and `rehash` need the same normalized, recursively sorted config/schema/ignore set and complete spec inventory.
- `filter_by_status` reads only the frontmatter section (up to the closing `---`) to avoid re-reading full file bodies that callers parse again later.

## Files to Read First

- `src/commands/mod.rs` — the module itself (all shared functions + submodule re-exports)
- `src/main.rs` — dispatches every `Command` variant into these submodules; also home to the `compute_exit_code` unit tests
- `src/validator.rs` — `find_spec_files`, `validate_spec` (the core called by `run_validation`)
- `src/config.rs` — `load_config` used by `load_and_discover`

## Current Status

Fully implemented and stable. Snapshot-aware orchestration is covered through issue #429 CLI integration tests; `compute_exit_code` remains unit-tested in `src/main.rs`. `mod.rs` itself has no `#[cfg(test)]` module.

## Notes

- This module orchestrates library modules rather than containing domain logic.
- `filter_by_status` warns on unrecognized status strings so typos don't silently filter nothing.
