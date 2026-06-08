---
spec: cmd_init.spec.md
---

## Tasks

- (none open) — module is implemented and well covered.

## Done

- [x] `cmd_init` writes a default `specsync.json` with auto-detected `sourceDirs`.
- [x] Refuses to overwrite an existing `specsync.json` or `.specsync.toml`.
- [x] `ensure_hashes_gitignored` adds `.specsync/hashes.json` to root `.gitignore`, idempotently, with non-fatal warning on failure.
- [x] Inline unit tests for `ensure_hashes_gitignored`: `adds_entry_to_missing_gitignore`, `is_idempotent_when_entry_already_present`, `errors_when_gitignore_path_is_unwritable`.
- [x] Integration coverage for config creation, no-overwrite, and source-dir auto-detection (src/lib/multi/fallback/node_modules-ignore) plus the MCP `init` tool.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
