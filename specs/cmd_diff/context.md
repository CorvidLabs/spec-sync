---
spec: cmd_diff.spec.md
---

## Key Decisions

- Export deltas are computed against the **spec body's documented symbols**, not against the base revision's source tree. `new_exports` = currently-exported symbols missing from the spec; `removed_exports` = spec symbols no longer exported. This keeps the command dependency-free of `git show` and frames drift as "spec vs. reality".
- The git changed-file scan uses `git diff --name-only --end-of-options <base>`. `--end-of-options` is a deliberate hardening step so a base ref like `--upload-pack=...` can never be interpreted as a git flag.
- PR base auto-detection: when the caller passes the default `HEAD`, `detect_pr_base()` reads `GITHUB_EVENT_NAME` and `GITHUB_BASE_REF` and rewrites the base to `origin/<base_ref>` for `pull_request`/`pull_request_target` events. An explicit `--base` always wins.
- A spec is reported when either its tracked files changed or the `.spec.md` itself changed (`spec_modified`), so spec-only edits still surface.

## Files to Read First

- `src/commands/diff.rs` — the whole command, including the `DiffEntry` struct and `detect_pr_base()`.
- `src/output.rs` — `print_diff_markdown`, used for Markdown/Github output.
- `src/exports/mod.rs` — `scan_exported_symbols_full` / `has_configured_extension`, the source of current exports. The per-language extractors live beside it under `src/exports/`; there is no `src/exports.rs`.
- `src/parser.rs` — `parse_frontmatter` and `get_spec_symbols` (spec-documented API surface).

## Current Status

Implemented and stable. Covered by nine integration tests — seven in `tests/integration/commands.rs`, two in `tests/integration/regression_w1.rs`. No inline unit tests in `diff.rs`.

## Notes

- Output formats: Json and Markdown/Github render structured reports; Text/Table/Csv share a human-readable path that, when no specs match, lists changed source files not covered by any spec.
- This module orchestrates library modules (exports, output, parser) rather than containing domain logic.
