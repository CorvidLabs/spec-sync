---
spec: cmd_init.spec.md
---

## Key Decisions

- Both `specsync.json` and `.specsync.toml` are checked before writing; either one means "already initialized" and init no-ops. This avoids clobbering a TOML-configured project.
- The default config is built as a `serde_json::Value` and pretty-printed with a trailing newline, so the on-disk file is stable and diff-friendly.
- `ensure_hashes_gitignored` returns `Result<bool, String>` (not `io::Error`) so the caller can print a plain warning; the io error is mapped to a "Failed to update .gitignore" string. A `.gitignore` write failure is intentionally **non-fatal** — the config is already written and the cache being un-ignored is a minor issue.
- Only the repository-root `.gitignore` is edited (a `# spec-sync hash cache` comment plus the `.specsync/hashes.json` entry).

## Files to Read First

- `src/commands/init.rs` — `cmd_init`, `ensure_hashes_gitignored`, and the three inline unit tests.
- `src/config.rs` — `detect_source_dirs` (the auto-detection logic exercised heavily by integration tests).

## Current Status

Implemented and stable. Strong coverage: three inline unit tests on the gitignore helper plus several integration tests on config creation and source-dir detection.

## Notes

- `ensure_hashes_gitignored` is `pub`, so it is callable outside `cmd_init` (e.g. rehash flows) and is unit-tested directly.
- Part of the command layer — orchestrates `config::detect_source_dirs` rather than containing domain logic.
