---
spec: cmd_init.spec.md
---

## Key Decisions

- Current and legacy config locations are checked before writing; an existing configuration produces a migration/current-layout hint without being overwritten.
- Fresh projects write deterministic `.specsync/config.toml`, `.specsync/version`, and `.specsync/sdd.json` files.
- `ensure_hashes_gitignored` returns `Result<bool, String>` (not `io::Error`) so the caller can print a plain warning; the io error is mapped to a "Failed to update .gitignore" string. A `.gitignore` write failure is intentionally **non-fatal** — the config is already written and the cache being un-ignored is a minor issue.
- The repository-root `.gitignore` receives the hash-cache entry, while `.specsync/.gitignore` owns local lifecycle lock and transaction-journal entries.

## Files to Read First

- `src/commands/init.rs` — `cmd_init`, `ensure_hashes_gitignored`, and the inline unit tests.
- `src/config.rs` — `detect_source_dirs` (the auto-detection logic exercised heavily by integration tests).

## Current Status

Implemented for 6.0. Fresh projects write SDD off, do not detect or run project test commands, and do not start a first-change interview. `specsync check` is the product. Enable the change workflow with `specsync change adopt`.

## Notes

- `ensure_hashes_gitignored` is `pub`, so it is callable outside `cmd_init` — `commands::migrate` is the one other caller — and is unit-tested directly. `rehash` does not call it.
- Part of the command layer — orchestrates `config::detect_source_dirs` rather than containing domain logic.
