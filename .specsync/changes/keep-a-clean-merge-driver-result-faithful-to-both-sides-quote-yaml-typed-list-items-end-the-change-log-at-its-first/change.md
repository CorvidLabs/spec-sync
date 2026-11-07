---
id: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
state: implementing
type: bug_fix
base_commit: ab6882464fadb1bdfc71416d729534323e5a3836
---

# Keep a clean merge-driver result faithful to both sides: quote YAML-typed list items, end the change log at its first blank line, merge version comments three ways, and stop renumbering at u64::MAX (#870 #872 #875 #876)

## Intent

Keep a clean merge-driver result faithful to both sides: quote YAML-typed list items, end the change log at its first blank line, merge version comments three ways, and stop renumbering at u64::MAX (#870 #872 #875 #876)

## Affected Canonical Specs

- `merge`

## Acceptance Criteria

- #872 (MERGE-5.a): after a union of `files`, `db_tables` or `depends_on`, an item YAML would read as a number, boolean or null (`true`/`false`/`yes`/`no`/`on`/`off`/`null`/`~` in any case, integers, floats, hex and octal, `.inf`, `.nan`) is written double-quoted and stays a string, while a plain path stays plain.
- #875 (MERGE-5.b): the first blank line after the change-log rows ends the table and stays in its suffix; prose or a second table after it comes out byte-for-byte, and that table's rows are never sorted, renumbered or deduplicated as change-log rows.
- #876 (MERGE-5.c): when both sides bump a numeric `version:`, the text after the number (quote and ` # comment`) merges three ways: kept when both sides agree, the other side's when one side kept the ancestor's, and a conflict when both changed it differently. The output is byte-identical in either merge order.
- #870 (MERGE-5.d): renumbering a new change-log `Version` cell uses a checked increment; when no higher u64 exists, the table stays a conflict, with no panic in debug and no wrap to 0 in release.
- Each criterion has a test that fails on main first. Existing merge-driver tests are unchanged, and a one-sided edit is still returned byte-for-byte (MERGE-4).

## No-spec Rationale

Not applicable
