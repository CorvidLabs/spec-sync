---
spec: cmd_new.spec.md
---

## Key Decisions

- **Scaffold, don't author**: `cmd_new` writes frontmatter and section skeletons but leaves prose, invariants, and dependency descriptions to the author. Public API rows are review prompts, not finished docs.
- **Auto-detect sources two ways**: a `src/<module>/` directory (walked recursively) and a top-level `src/<module>.<ext>` file are both treated as the module's sources, filtered by configured `source_extensions`.
- **Pre-populate exports**: exported symbols from detected sources are pulled via `exports::get_exported_symbols`, de-duplicated, and seeded into the Public API table so the author starts from the real surface area.
- **Never clobber**: an existing target spec aborts with exit 1 — creation is non-destructive.
- **No chrono dependency**: dates come from the in-module `chrono_lite_today()` helper to keep the binary dependency-light and cross-platform.

## Files to Read First

- `src/commands/new.rs` — the command itself: `cmd_new`, `detect_module_sources`, and `chrono_lite_today`.
- `src/generator.rs` — `generate_companion_files_for_spec`, invoked under `--full`.
- `src/exports.rs` — `get_exported_symbols` and `has_extension`, used for source/export detection.

## Current Status

Stable and implemented. Integration tests cover basic creation, source auto-detection, no-match guidance, and
module-name safety. The command module has no inline `#[cfg(test)]` module; explicit `--full` and refuse-overwrite
integration fixtures remain open.

## Notes

- This is a command-layer module: it orchestrates `config`, `exports`, and `generator` rather than holding domain logic.
- `depends_on` is always emitted empty; imports are not analyzed to infer dependencies.
