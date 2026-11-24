# Lesson bundle: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A clean merge-driver union keeps a quoted YAML date quoted
- **Kind**: BugFix
- **Specs**: merge
- **Paths**: src/merge_driver.rs
- **Acceptance**: A quoted YYYY-MM-DD list item stays a quoted string after a clean union of files in either merge order; a full timestamp stays quoted; a path holding a date stays plain (red then green)

## Evidence

- Verification commit: `6e8b969d2fa2b4c396c689a603a28bd9de70f00d`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec merge`

## From the change's context.md

# Context

Issue #1012, a Codex P2 on #1006 tracked under Leif's two-round cap. When a list union holds a
quoted item such as `"2026-11-07"`, `render_list_item` treats it as plain because
`yaml_reads_as_non_string` covers booleans, nulls and numbers but not timestamps, so the merged
list writes `2026-11-07` unquoted. YAML 1.1 reads that as a timestamp, so a clean merge changes
the item's type, against REQ-merge-006 ("means what they meant").

Only the date-only form gets through `render_list_item`'s plain-character check: a full
timestamp holds `:` or a space, and so does a YAML 1.1 sexagesimal number, so both are already
quoted. The fix adds the YAML 1.1 date form `[0-9]{4}-[0-9]{2}-[0-9]{2}` to
`yaml_reads_as_non_string`. No requirement text changes.

## From the change's testing.md

# Testing

- Unit `quoted_yaml_timestamps_stay_strings_after_a_union` in `src/merge_driver.rs`: a quoted
  `2026-11-07` stays quoted after a clean union in both merge orders (fails on main: it comes out
  unquoted); a full timestamp stays quoted; `docs/2026-11-07.md` stays plain.
- The existing #872 typed-item and merge-driver tests still pass.

## Where these lessons go

- `specs/merge/merge.3md`
