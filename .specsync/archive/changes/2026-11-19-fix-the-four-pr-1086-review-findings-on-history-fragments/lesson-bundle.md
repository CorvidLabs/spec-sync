# Lesson bundle: fix-the-four-pr-1086-review-findings-on-history-fragments

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Fix the four PR 1086 review findings on history fragments
- **Kind**: BugFix
- **Specs**: change, hash_cache
- **Paths**: src/change/, src/change_tests/, src/hash_cache.rs, tests/integration/spec_history.rs, specs/change/, specs/hash_cache/, site/src/content/docs/deltas.md, CHANGELOG.md
- **Acceptance**: Every valid change id round-trips through its fragment; a fragment swapped for a same-bytes symlink is re-validated instead of replaying a cached pass; a delta item that targets the Change Log is refused at approve, check, verify and accept; a reformatted legacy row plus a fragment is refused as a double record and a row for one-session does not count for session

## Evidence

- Verification commit: `12a7e509b3b5492c2df49ea3231775f8c85f6b82`
- Base commit: `069e6eaa9565a8ebf4837b0fb1f4f95c51234d0d`
- Verified by: `specsync check --spec change --spec hash_cache`

## From the change's context.md

# Context

Follow-up to #1084 / PR #1086. Codex left four findings on #1086. corvid-agent's approval judged none blocking, and Leif chose "Merge, fix all 4 in a follow-up before release", relayed by hc t4c on 2026-11-18. #1086 merged as 3bebe8b0. Its four threads were answered with a link to this PR (#1088) and resolved.

The tests and fixes were written before this change was opened, so #1088 could be linked from #1086's threads before #1086 merged. Each test failed on 979c3fe9, the #1086 tip. The change was then opened on the branch rebased onto 3bebe8b0.

## From the change's design.md

# Design

- **Quote.** `history_fragment_content` writes `change: "<id>"` with `\"` and `\\` escapes; `validate_change_id` already refuses control characters and `\`. `yaml_scalar` reads double-quoted (only `\"` and `\\` escapes), single-quoted (`''`) or plain values.
- **Entry types.** `validation_input_digest_for_build` frames `<path>=<file|directory|symlink|other|missing>` for `history/` and each entry, sorted, through `symlink_metadata`, beside the paths whose bytes it already hashes.
- **Change Log items.** `refuse_change_log_delta_items` runs beside `validate_delta_files` in approve, materialize, verify and accept. It is not in the project audit, so an archived change that once carried such an item is not re-judged.
- **One matcher.** `changelog_records_change` now requires that the character before `<id>:` is not alphanumeric, `-`, `_` or `.`; `scan_history_fragments` uses it for the double record, and the narrower `changelog_row_names_change` is gone.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-159 | `change::tests::a_delta_that_targets_the_change_log_is_refused`, `change::tests::one_boundary_aware_matcher_finds_a_change_s_row`, `change::tests::a_double_record_names_the_change_at_an_id_boundary`, and the `spec_history` integration tests |
| REQ-change-160 | `change::tests::a_fragment_quotes_its_change_id_so_every_valid_id_round_trips`, `change::tests::a_written_history_fragment_parses_back` |
| REQ-hash-cache-006 | `hash_cache::tests::a_snapshot_is_not_replayed_over_a_fragment_swapped_for_a_symlink`, `hash_cache::tests::a_snapshot_is_not_replayed_over_a_changed_history_fragment` |

The four new tests failed on 979c3fe9 (the #1086 tip). Locally: the verify lane and the pre-push gate.

## Where these lessons go

- `specs/change/change.3md`
- `specs/hash_cache/hash_cache.3md`
