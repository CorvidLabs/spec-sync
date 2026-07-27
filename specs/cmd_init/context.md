---
spec: cmd_init.spec.md
---

## Key Decisions

- Current and legacy config locations are checked before writing; an existing configuration produces a migration/current-layout hint without being overwritten.
- Fresh projects write deterministic `.specsync/config.toml`, `.specsync/version`, and `.specsync/sdd.json` files.
- `ensure_hashes_gitignored` returns `Result<bool, String>` (not `io::Error`) so the caller can print a plain warning; the io error is mapped to a "Failed to update .gitignore" string. A `.gitignore` write failure is intentionally **non-fatal** — the config is already written and the cache being un-ignored is a minor issue.
- The repository-root `.gitignore` receives the hash-cache entry, while `.specsync/.gitignore` owns local lifecycle lock and transaction-journal entries.
- `init --repair` is additive: it validates the selected config first, then restores only missing support artifacts. Existing config, specs, policy, and ignore files are not regenerated.
- Expected directory/file topology is preflighted before fresh creation so a blocking file or symlink cannot leave a partially initialized layout.
- An initialized ancestor is reported as an error before a nested `.specsync` can be created.
- Every output format is rendered from one outcome record; JSON is a single parseable value and non-text formats skip interactive prompts.

## Files to Read First

- `src/commands/init.rs` — `cmd_init`, `ensure_hashes_gitignored`, and the three inline unit tests.
- `src/config.rs` — `detect_source_dirs` (the auto-detection logic exercised heavily by integration tests).

## Current Status

Implemented for 5.0. Fresh projects enable SDD, repairs are additive and config-checked, fallback detection is explicit, nested projects are rejected, and structured output remains deterministic.

## Notes

- `ensure_hashes_gitignored` is `pub`, so it is callable outside `cmd_init` (e.g. rehash flows) and is unit-tested directly.
- Part of the command layer — orchestrates `config::detect_source_dirs` rather than containing domain logic.
