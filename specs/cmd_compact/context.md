---
spec: cmd_compact.spec.md
---

## Key Decisions

- Thin command wrapper: load config, resolve `specs_dir`, call `compact::compact_changelogs`, format output. No trimming logic here.
- `--keep` is passed straight through; the wrapper only flips the printed verb ("would compact" vs "compacted") and prints the banner.
- Empty result is a success case ("No changelogs need compaction (all within limit).") with an early return — not an error.
- Per-spec output reports both `removed` and the surviving `compacted_entries` count so reviewers can sanity-check the keep limit.
- Singular/plural labels derive from each reported count; the kept count is ordinary rows only.
- `OutputFormat::Json` produces one document and expresses dry-run truth through separate `would_change` and `applied` booleans.
- `OutputFormat::Markdown` and `Github` share the PR-suitable table renderer; text-like formats preserve the established terminal output.
- Structured renderers normalize Windows separators to `/` without corrupting literal Unix backslashes.
- Markdown/GitHub uses one sanitized code element: dynamic-backtick spans for ordinary paths and entity-safe HTML code for literal-pipe paths, preserving every legal Unix backslash parity; JSON retains machine-readable paths and reports complete/partial operation truth.
- The command renders the full typed compact report and exits 1 when any operation fails.

## Files to Read First

- `src/commands/compact.rs` — the command wrapper (this module)
- `src/compact.rs` — `compact_changelogs` + `CompactResult { spec_path, compacted_entries, removed }`, where the changelog-table trimming lives
- `src/config.rs` — `load_config` / `specs_dir` resolution

## Current Status

Text, JSON/`--json`, Markdown/GitHub, hostile structured paths, literal Unix backslashes, deterministic late-publish partial truth, dry-run, and repeated-run behavior are covered end to end for issue #417.

## Notes

- `CompactResult.spec_path` is repo-relative; Windows separators normalize while Unix literal backslashes survive.
- Markdown table paths sanitize controls/bidi characters, entity-encode table pipes when needed, and otherwise use a delimiter longer than every embedded backtick run.
