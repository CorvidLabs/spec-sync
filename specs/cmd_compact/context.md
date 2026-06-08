---
spec: cmd_compact.spec.md
---

## Key Decisions

- Thin command wrapper: load config, resolve `specs_dir`, call `compact::compact_changelogs`, format output. No trimming logic here.
- `--keep` is passed straight through; the wrapper only flips the printed verb ("would compact" vs "compacted") and prints the banner.
- Empty result is a success case ("No changelogs need compaction (all within limit).") with an early return — not an error.
- Per-spec output reports both `removed` and the surviving `compacted_entries` count so reviewers can sanity-check the keep limit.

## Files to Read First

- `src/commands/compact.rs` — the command wrapper (this module)
- `src/compact.rs` — `compact_changelogs` + `CompactResult { spec_path, compacted_entries, removed }`, where the changelog-table trimming lives
- `src/config.rs` — `load_config` / `specs_dir` resolution

## Current Status

Implemented and stable. The `compact` delegate is unit-tested (trim, no-op when under limit, three-column tables); the wrapper itself has no inline tests (output formatting only).

## Notes

- `CompactResult.spec_path` is already repo-relative; the wrapper prints it verbatim.
