---
change: CHG-0156-the-reopen-then-close-guard-must-be-pinned-by-tests-not-only-by-a-drill
artifact: testing
---

# Testing

Two distinct removals, verified in a **separate clone** rather than by reverting in place.

| binary | round-trip | third-location | deletion |
|---|---|---|---|
| `main` | ok | ok | ok |
| archive-to-active term removed (the original #540 defect) | **FAILED** | ok | ok |
| guard deleted outright (the shim that scores 12/0/0 on drill 049) | ok | **FAILED** | **FAILED** |

The two failure modes land on **different** tests, which is the point. Removing a direction makes
the guard stricter; deleting the guard makes it absent. A single assertion cannot see both, which
is why drill 049 — asserting only `rc=0`, `state=archived`, `archives=1` — sees neither.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-078 | Reverting #540's archive-to-active term fails `scoped_review_evidence_may_move_between_a_change_s_two_homes_in_either_direction` while the other two still pass; deleting the guard outright passes that one and fails `scoped_review_evidence_moved_to_a_third_location_is_refused` and `scoped_review_evidence_may_not_be_deleted`. Both measured against binaries built from a separate clone. Before this change, reverting #540 left the whole suite green and `grep -rn 'moved evidence outside finalization' src/` returned one hit, in the product |
