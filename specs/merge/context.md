---
spec: merge.spec.md
---

## Key Decisions

- **Section-aware strategies**: `resolve_conflict` dispatches on the enclosing section (last `## ` heading, computed via `detect_section` from text emitted so far). Frontmatter (before any heading) unions known lists, takes the maximum numeric version, and rejects ambiguous scalars; `## Change Log` merges rows chronologically; any section that is *pure table rows* gets a lossless key-based union; everything else (prose) is left for manual resolution.
- **Exact region parsing, not regex precedence**: `parse_conflict_regions` accepts exact `<<<<<<< / ||||||| / ======= / >>>>>>>` forms and separates HEAD, base, and incoming regions. Base content is excluded from auto-resolution inputs; orphan, nested, duplicate, incomplete, and lookalike markers keep the complete file manual.
- **Conservative YAML subset plus checked validation**: `parse_yaml_fields` distinguishes null scalars from empty lists and handles only top-level key/scalar and known-list shapes. Nested mappings remain manual. Every candidate output must contain frontmatter and is checked by the shared YAML parser for duplicate keys and by canonical required-field/status rules.
- **No ambiguous winner**: numeric versions use the maximum value and known frontmatter lists are unioned. Divergent or one-sided non-version scalars, nonnumeric versions, same-key table rows, and table header/separator hunks remain manual; no branch is silently selected.
- **All-or-nothing persistence**: a file is written only after every hunk resolves and frontmatter validates. Auto-resolvable hunks are reported but not persisted when any hunk remains manual.
- **Truthful persistence wording**: dry-run candidates remain `Auto-resolvable`; details become `Auto-resolved` only after the complete file is successfully written.
- **Conservative fallbacks**: when `git diff` fails in git mode, `detect_conflicted_specs` returns an empty list (no specs processed); unreadable files become `Manual`; a still-conflicted result keeps its markers.

## Files to Read First

- `src/merge.rs` — entire module: `merge_specs` (driver), `resolve_spec_conflicts` (orchestrator), `parse_conflict_regions`, `resolve_conflict` and the three strategy functions (`resolve_changelog_conflict`, `resolve_table_conflict`, `resolve_frontmatter_conflict`).

## Current Status

CHG-0066 implementation is under verification after independent acceptance and adversarial review. Public API remains `merge_specs`, `has_conflict_markers`, `print_results`, `results_to_json`, plus `MergeResult`/`MergeStatus`.

## Notes

- Depends on `parser::parse_frontmatter` and `parse_checked_issue_references` for post-resolution structural/YAML validation and `validator::find_spec_files` for all-files scan.
- Changelog sorting assumes ISO `YYYY-MM-DD` dates so lexicographic order is chronological.
- `print_results` skips `Clean` entries; `results_to_json` includes all three statuses.
