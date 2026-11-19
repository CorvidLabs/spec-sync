---
change: fix-the-four-pr-1086-review-findings-on-history-fragments
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-159 | `change::tests::a_delta_that_targets_the_change_log_is_refused`, `change::tests::one_boundary_aware_matcher_finds_a_change_s_row`, `change::tests::a_double_record_names_the_change_at_an_id_boundary`, and the `spec_history` integration tests |
| REQ-change-160 | `change::tests::a_fragment_quotes_its_change_id_so_every_valid_id_round_trips`, `change::tests::a_written_history_fragment_parses_back` |
| REQ-hash-cache-006 | `hash_cache::tests::a_snapshot_is_not_replayed_over_a_fragment_swapped_for_a_symlink`, `hash_cache::tests::a_snapshot_is_not_replayed_over_a_changed_history_fragment` |

The four new tests failed on 979c3fe9 (the #1086 tip). Locally: the verify lane and the pre-push gate.
