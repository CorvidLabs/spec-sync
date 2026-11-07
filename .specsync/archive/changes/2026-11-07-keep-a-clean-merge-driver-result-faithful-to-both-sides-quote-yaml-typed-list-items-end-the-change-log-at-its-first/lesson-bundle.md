# Lesson bundle: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep a clean merge-driver result faithful to both sides: quote YAML-typed list items, end the change log at its first blank line, merge version comments three ways, and stop renumbering at u64::MAX (#870 #872 #875 #876)
- **Kind**: BugFix
- **Specs**: merge
- **Paths**: src/merge_driver.rs, specs/merge, hi/merge.md, INTENT.md, CHANGELOG.md
- **Acceptance**: #872 (MERGE-5.a): after a union of `files`, `db_tables` or `depends_on`, an item YAML would read as a number, boolean or null (`true`/`false`/`yes`/`no`/`on`/`off`/`null`/`~` in any case, integers, floats, hex and octal, `.inf`, `.nan`) is written double-quoted and stays a string, while a plain path stays plain.
- **Acceptance**: #875 (MERGE-5.b): the first blank line after the change-log rows ends the table and stays in its suffix; prose or a second table after it comes out byte-for-byte, and that table's rows are never sorted, renumbered or deduplicated as change-log rows.
- **Acceptance**: #876 (MERGE-5.c): when both sides bump a numeric `version:`, the text after the number (quote and ` # comment`) merges three ways: kept when both sides agree, the other side's when one side kept the ancestor's, and a conflict when both changed it differently. The output is byte-identical in either merge order.
- **Acceptance**: #870 (MERGE-5.d): renumbering a new change-log `Version` cell uses a checked increment; when no higher u64 exists, the table stays a conflict, with no panic in debug and no wrap to 0 in release.
- **Acceptance**: Each criterion has a test that fails on main first. Existing merge-driver tests are unchanged, and a one-sided edit is still returned byte-for-byte (MERGE-4).

## Evidence

- Verification commit: `6c7d45834e6e990f6a958c7570282e37c3f04111`
- Base commit: `ab6882464fadb1bdfc71416d729534323e5a3836`
- Verified by: `specsync check --spec merge`

## From the change's context.md

# Context

- Issues #870, #872, #875 and #876 were filed from the #865 review, which added the git merge driver (`src/merge_driver.rs`, `merge_contract`). The fix rounds there stopped after the third round, so these were tracked instead. They are the triage group G-merge-render (`_audits/specsync-triage-2026-11-06.md`).
- In each one the driver reports a clean merge whose result is wrong:
  - #872: a quoted YAML-typed list item is written plain, so YAML reads it as a number, boolean or null.
  - #875: a blank line after the change-log table is dropped, and rows of a following table can be absorbed and renumbered.
  - #876: version comments are taken from `ours` only, so the result depends on merge direction and loses a comment.
  - #870: the change-log `Version` renumbering overflows at `u64::MAX`, panicking in debug and wrapping to 0 in release.
- HI: MERGE-5 and MERGE-5.a to 5.d, drafted from these issues. Leif approved them as drafted on 2026-11-06 ("Capture MERGE-5 + cases"), relayed by the orc (t18). MERGE-1 (either order) and MERGE-4 (no looser gates) also apply.
- REQ-merge-006 is assigned by the orc's registry.
- Red first: the four new tests in `src/merge_driver.rs` fail on main ab688246 for exactly the reasons in their issues. They are committed with this draft.
- Human gates: scope approval and implementation review come from Leif through the orc. This agent never records them as Leif.

## From the change's design.md

# Design

All changes are in `src/merge_driver.rs`; no public API changes.

- **#872, `render_list_item`.** Besides today's rule (plain only for safe path characters), it double-quotes an item YAML would read as something other than a string:
  - `true/false/yes/no/on/off/y/n/null/~` in any case;
  - anything Rust parses as `f64` (integers, floats, exponents, `inf`, `nan`, a leading sign);
  - `.inf` / `.nan` with an optional sign;
  - `0x`/`0o`/`0b` integers;
  - digit-and-underscore numbers such as `1_000` (YAML 1.1).
  Over-quoting is always safe because the value stays a string. These three fields (`files`, `db_tables`, `depends_on`) are string lists, so a plain `123` written quoted gets its intended type.
- **#875, `parse_changelog_table`.** The first blank line after the separator ends the rows, and it and everything after it become the suffix. Today an interior blank is skipped unless only blanks follow. The suffix is already merged as a text region, so text after the table merges like any other text: a one-sided edit is clean, and overlapping two-sided edits conflict.
- **#876, `merge_version_field`.** The decoration after the number merges three ways (see the rules below). The decoration is the text after the closing quote, or from ` #` for a plain scalar. If the merge conflicts, the field is a conflict.
  - The rendered line takes its decoration from the side whose decoration won.
  - The early `Keep(theirs)` shortcut applies only when that side's decoration is the merged one, so a one-sided edit is still returned byte for byte.
  - Quote style divergence stays a conflict, as today (invariant 22).
- **Three-way decoration rules (#876):**
  - Both sides equal: keep it.
  - One side kept the ancestor's: take the other side's.
  - Both changed it differently: conflict.
- **#870, `union_changelog_tables`.** `next = next.checked_add(1)`; on `None` it returns `changelog_conflict(ours, theirs)`, the same conflict as a non-integer cell.
- Rejected: an explicit merge of two different comments (for example `# A; B`). It invents text neither side wrote, and MERGE-5 prefers a conflict to a guess.
- Rejected: quoting every list item after a union. It churns every spec's formatting on each merge.

## From the change's testing.md

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

## Where these lessons go

- `specs/merge/merge.3md`
