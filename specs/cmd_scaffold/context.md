---
spec: cmd_scaffold.spec.md
---

## Key Decisions

- Two entry points share one file: `cmd_add_spec` (fixed built-in template, no flags) and `cmd_scaffold` (optional `--dir` and `--template`, plus registry auto-registration).
- Neither command uses AI — spec bodies come from inline string templates or the generator. AI generation is the separate `generate` command.
- An existing `*.spec.md` is never overwritten; both commands instead backfill missing companions and return early, so re-running is safe and idempotent for companions.
- Companion emission is delegated to `generator` (`generate_companion_files_for_spec` / `generate_companion_files_from_template`), keeping template content in one place.
- Registry auto-registration is opt-in by file presence: it only fires when `specsync-registry.toml` already exists at the repo root.

## Files to Read First

- `src/commands/scaffold.rs` — primary source (both `cmd_add_spec` and `cmd_scaffold`)
- `src/generator.rs` — `find_files_for_module`, `generate_spec`, companion generators
- `src/registry.rs` — `register_module` for auto-registration
- Source auto-detection is NOT done here: `scaffold.rs` never touches `exports`, and both entry points delegate detection to `generator::find_files_for_module`

## Current Status

Fully implemented and stable. One inline unit test (`add_spec_omits_module_javascript_test_sources`) pins test-file exclusion; companion-generation behavior is covered by integration tests (`generate_creates_companion_files`, `companion_files_not_overwritten_on_regenerate`).

## Notes

- Part of the command layer: orchestrates `config`, `generator`, and `registry` rather than holding domain logic.
- Source-file paths in generated frontmatter are normalized to forward slashes for cross-platform manifests.
