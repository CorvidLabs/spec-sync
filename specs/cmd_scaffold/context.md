---
spec: cmd_scaffold.spec.md
---

## Key Decisions

- Two entry points share one file: `cmd_add_spec` (no flags) and `cmd_scaffold` (optional `--dir` and `--template`, plus registry auto-registration).
- Neither command uses AI — all spec bodies come from the shared deterministic generator.
- An existing `*.spec.md` is never overwritten; both commands instead backfill missing companions and return early, so re-running is safe and idempotent for companions.
- Companion emission is delegated to `generator` (`generate_companion_files_for_spec` / `generate_companion_files_from_template`), keeping template content in one place.
- Registry auto-registration is opt-in by file presence: it only fires when `specsync-registry.toml` already exists at the repo root.

## Files to Read First

- `src/commands/scaffold.rs` — primary source (both `cmd_add_spec` and `cmd_scaffold`)
- `src/generator.rs` — `find_files_for_module`, `generate_spec`, companion generators
- `src/registry.rs` — `register_module` for auto-registration
- `src/exports.rs` — `has_extension` used during source auto-detection

## Current Status

Fully implemented and stable. Issue #421 empty-file and API-population regressions are covered directly; companion generation remains covered by the existing regeneration fixtures.

## Notes

- Part of the command layer: orchestrates `config`, `generator`, `registry`, and `exports` rather than holding domain logic.
- Source-file paths in generated frontmatter are normalized to forward slashes for cross-platform manifests.
