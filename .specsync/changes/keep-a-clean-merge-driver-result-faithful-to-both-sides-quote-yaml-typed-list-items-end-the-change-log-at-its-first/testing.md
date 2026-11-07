---
change: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
artifact: testing
---

# Testing

## Plan

New unit tests in `src/merge_driver.rs`; each runs both merge orders:

- `yaml_typed_list_items_stay_strings_after_a_union` (#872): 16 typed values in `files`, plus `db_tables: ["true"]` and `depends_on: ["null"]`. Each must come out double-quoted, while `src/a.rs` stays plain.
- `text_after_the_change_log_table_survives_a_row_union` (#875): prose and then a second table after the change log, and a second table directly after a blank line. The merged file must equal the expected text byte for byte.
- `version_comments_merge_three_ways_in_either_order` (#876):
  - a comment on one side only survives (`version: 7 # B`);
  - the same comment on both sides is kept (`version: 7 # A`);
  - two different comments are a conflict that shows both.
- `changelog_version_renumbering_stops_at_u64_max` (#870): an ancestor row at `u64::MAX` and a new row on each side. The result must be a conflict with no `| 0 |`, and no panic.
- Mutants, one per fix, each restored byte for byte. The matching test must fail.
- The existing 23 merge-driver tests stay unchanged and green.
- `fledge lanes run verify` (pinned 1.89) and pre-push.

## Results (local macOS, pinned rustc 1.89.0)

- Red first, on main ab688246 plus the four new tests only: 23 passed, 4 failed.
  - `changelog_version_renumbering_stops_at_u64_max` panicked with "attempt to add with overflow" (`src/merge_driver.rs:537`).
  - `text_after_the_change_log_table_survives_a_row_union`: the merged text differs from both sides' unchanged tail.
  - `version_comments_merge_three_ways_in_either_order`: `version: 7 # B` missing in one order (the comment was dropped).
  - `yaml_typed_list_items_stay_strings_after_a_union`: `123` was written unquoted.
