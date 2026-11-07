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

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-merge-006 | unit tests `yaml_typed_list_items_stay_strings_after_a_union`, `text_after_the_change_log_table_survives_a_row_union`, `version_comments_merge_three_ways_in_either_order` and `changelog_version_renumbering_stops_at_u64_max` in `src/merge_driver.rs` (failing first at `3bbcfc26`; each fix's mutant killed) |

## Results (local macOS, pinned rustc 1.89.0)

- Red first, on main ab688246 plus the four new tests only: 23 passed, 4 failed.
  - `changelog_version_renumbering_stops_at_u64_max` panicked with "attempt to add with overflow" (`src/merge_driver.rs:537`).
  - `text_after_the_change_log_table_survives_a_row_union`: the merged text differs from both sides' unchanged tail.
  - `version_comments_merge_three_ways_in_either_order`: `version: 7 # B` missing in one order (the comment was dropped).
  - `yaml_typed_list_items_stay_strings_after_a_union`: `123` was written unquoted.
- Two expectations in the draft's tests were corrected after the fix, without weakening either. Main's code still fails both corrected tests (below).
  - The #872 test now asserts the exact sorted lists. The union sorts items, so `"true"` sits between `orders` and `users`.
  - The #876 "same comment" case now uses two different changes. Byte-identical sides are one change, not two bumps. The red run never reached that case.
- Green: all 113 tests matching `merge` pass, including the 27 merge-driver tests.
- Mutants, each restored byte for byte (sha before and after `046f67d754e7`):

| Mutant | Change | Test that fails |
| --- | --- | --- |
| M870 | renumbering uses an unchecked `+= 1` | `changelog_version_renumbering_stops_at_u64_max` |
| M872 | `render_list_item` ignores YAML-typed items | `yaml_typed_list_items_stay_strings_after_a_union` |
| M875 | a blank line inside the change log is skipped again | `text_after_the_change_log_table_survives_a_row_union` |
| M876-side | the version line takes `ours`' decoration again | `version_comments_merge_three_ways_in_either_order` |
| M876-conflict | two different version notes no longer conflict | `version_comments_merge_three_ways_in_either_order` |
| MAIN | main's code (ab688246) with the corrected tests | all four new tests; the other 23 pass |

## Codex review of #1006 at 6821fb7f

- P2 "Quote YAML floats that combine underscores and exponents": fixed. The #872 test gained `1_000`, `1_000.5e+2`, `+1_000.5` and `0x_1F`. It failed on `1_000.5e+2` first. `yaml_reads_as_non_string` now reads an item with underscores as a number when it parses as `f64` once they are removed; that replaces the narrower digits-and-dots rule.
- P2 "Make differing-comment conflicts independent of merge order": no code change. Every conflict the driver writes labels its halves `ours` (the branch merged into) and `theirs`, as git does. Sorting this one conflict's halves would put `theirs`' text under the `ours` label in one direction. The test now pins the property that holds: the two orders give the same two lines, each under its own label, with the halves mirrored. Invariant 22 now says a clean result is byte-identical in either order and a conflict is mirrored.
- After both: 113 tests matching `merge` pass, and clippy is clean.

