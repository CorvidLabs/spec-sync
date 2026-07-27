---
spec: merge.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/merge.rs` | `fledge run test -- merge::` | Issue #427 regressions for all supported list unions, numeric version syntax/max, divergent and one-sided scalars, exact/malformed/orphan/nested markers, header-only and separator hunks, null/list type preservation, nested YAML, required frontmatter, CRLF/final newline, accurate details, dry-run, and all-or-nothing writes |
| `tests/integration/commands.rs` | `fledge run test -- merge_issue_427` | CLI dry-run and real-run diff3/version behavior, persisted-resolution wording, and byte-identical mixed-manual persistence |

## Coverage Gaps

- Structured JSON dry-run schema is owned by issue #420 and its reporting PR; CHG-0066 keeps the existing public JSON shape.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Auto-resolve frontmatter list conflict | ours has `files: [a.rs, b.rs]` and theirs has `files: [b.rs, c.rs]` | `merge_specs` resolves the conflict | result is `files: [a.rs, b.rs, c.rs]` (union, sorted) |
| Auto-resolve changelog conflict | ours added a `2026-03-20` entry, theirs added a `2026-03-25` entry | `merge_specs` resolves the conflict | both entries appear in chronological order, status is `Resolved` |
| Prose conflict requires manual resolution | both sides modified the `## Purpose` section text | `merge_specs` encounters the conflict | conflict markers are preserved, status is `Manual` |
| Divergent API row | both sides modify the same Public API symbol differently | `merge_specs` encounters the conflict | status is `Manual`, both labels and the ambiguity are reported, and the file is unchanged |
| Diff3 table conflict | a lossless row union includes a `||||||| base` section | `merge_specs` resolves the conflict | HEAD and incoming rows remain; base marker/content does not leak |
| Dry run | conflicted spec files exist | `merge_specs(root, specs_dir, true, false)` is called | returns `MergeResult` entries with resolution details but does not modify files |
| Malformed marker or lossy parser boundary | orphan/nested/lookalike marker, table header/separator, null/list disagreement, nested YAML, or missing frontmatter appears | `merge_specs` encounters the file | status is `Manual` and disk bytes are unchanged |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec file unreadable | Marked as `Manual` with read error in details | `all_files_reports_unreadable_specs_as_manual` |
| `git diff` command fails (`all_files: false`) | `detect_conflicted_specs` returns an empty list — no specs are processed (no auto-fallback to scanning all files) | `git_discovery_failure_is_a_safe_noop` |
| Post-resolution frontmatter invalid | Marked `Manual`; original file remains unchanged | `duplicate_or_invalid_frontmatter_suppresses_otherwise_resolvable_writes` |
| Post-resolution frontmatter missing | Marked `Manual`; original file remains unchanged | `missing_frontmatter_prevents_resolvable_body_write` |
| Partially resolvable file | Report auto-resolvable and manual hunk counts; preserve the original file byte-for-byte | `partial_resolution_preview_keeps_all_or_nothing_write_boundary`, `merge_specs_leaves_partially_resolvable_file_untouched` |
| Malformed conflict block | Marked `Manual`; no write | `malformed_conflict_block_is_never_auto_resolved` |
| CRLF diff3 input | Fully resolved output uses CRLF without lone LF bytes | `resolved_diff3_file_preserves_crlf_line_endings` |
| Missing final newline | Fully resolved output remains unterminated | `fully_resolved_file_preserves_missing_final_newline` |
| Orphan/nested/lookalike markers | Marked `Manual`; on-disk file stays byte-identical | `orphan_marker_families_after_a_valid_hunk_block_every_write`, `nested_marker_diagnostics_keep_the_outer_side_labels`, `diff3_marker_requires_exact_seven_pipes_and_label_separator` |
| Header/separator or nested YAML in hunk | Marked `Manual`; no lossy reconstruction | `table_hunks_containing_headers_or_separators_remain_manual`, `table_header_only_hunks_remain_manual_and_byte_identical`, `nested_frontmatter_mapping_is_not_flattened` |
| YAML null versus empty list | Marked `Manual`; neither type is rewritten as the other | `unknown_empty_scalar_and_empty_list_are_not_equivalent` |
| Dry-run versus persisted wording | Preview details say `Auto-resolvable`; successful real writes say `Auto-resolved` | `merge_resolves_interior_field_conflict`, `merge_issue_427_diff3_max_version_and_dry_run_are_lossless` |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/merge.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
