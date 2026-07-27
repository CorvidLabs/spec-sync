---
spec: cmd_init.spec.md
---

## Tasks

- (none open) — module is implemented and well covered.

## Done

- [x] `cmd_init` writes the 5.0 `.specsync/config.toml` layout with auto-detected `source_dirs`, policy, and version stamp.
- [x] Refuses to overwrite existing current or legacy configuration.
- [x] `ensure_hashes_gitignored` adds `.specsync/hashes.json` to root `.gitignore`, idempotently, with non-fatal warning on failure.
- [x] Inline unit tests for `ensure_hashes_gitignored`: `adds_entry_to_missing_gitignore`, `is_idempotent_when_entry_already_present`, `errors_when_gitignore_path_is_unwritable`.
- [x] Integration coverage for config creation, no-overwrite, and source-dir auto-detection (src/lib/multi/fallback/node_modules-ignore) plus the MCP `init` tool.
- [x] Add `init --repair` with config preflight and additive support-file restoration.
- [x] Reject initialized ancestors and blocking layout topology before writes.
- [x] Emit truthful structured outcomes for creation, no-op, repair, and failure.
- [x] Preserve config/spec/root-ignore bytes across re-init and repair.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
