---
spec: merge.spec.md
---

## Key Decisions

- **Section-aware strategies**: `resolve_conflict` dispatches on the enclosing section (last `## ` heading, computed via `detect_section` from text emitted so far). Frontmatter (before any heading) unions lists and takes theirs for scalars; `## Change Log` merges rows chronologically; any section that is *pure table rows* gets a key-based table merge; everything else (prose) is left for manual resolution.
- **Region parsing, not regex**: `parse_conflict_regions` walks lines, peeling `<<<<<<< / ======= / >>>>>>>` blocks into `ours`/`theirs` strings and a `marker_label`. Clean text between blocks is preserved verbatim.
- **Zero-dependency YAML**: `parse_yaml_fields` handles the project's simple key/scalar and key/list subset directly rather than pulling in a YAML crate. List keys `files`, `db_tables`, `depends_on` are unioned and sorted.
- **Theirs-wins precedence**: for scalar frontmatter fields and generic-table row collisions, the incoming (theirs) value overrides ours, treating the merged-in branch as the newer change.
- **Conservative fallbacks**: when `git diff` fails in git mode, `detect_conflicted_specs` returns an empty list (no specs processed); unreadable files become `Manual`; a still-conflicted result keeps its markers.

## Files to Read First

- `src/merge.rs` — entire module: `merge_specs` (driver), `resolve_spec_conflicts` (orchestrator), `parse_conflict_regions`, `resolve_conflict` and the three strategy functions (`resolve_changelog_conflict`, `resolve_table_conflict`, `resolve_frontmatter_conflict`).

## Current Status

Stable and complete. Public API: `merge_specs`, `has_conflict_markers`, `print_results`, `results_to_json`, plus `MergeResult`/`MergeStatus`. Invoked by the `cmd_merge` subcommand.

## Notes

- Depends on `parser::parse_frontmatter` (post-resolution validation) and `validator::find_spec_files` (all-files scan).
- Changelog sorting assumes ISO `YYYY-MM-DD` dates so lexicographic order is chronological.
- `print_results` skips `Clean` entries; `results_to_json` includes all three statuses.
