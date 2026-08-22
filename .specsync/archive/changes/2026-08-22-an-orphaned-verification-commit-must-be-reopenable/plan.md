---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: plan
---

# Plan

1. Extract `accepted_evidence_is_anchored` and split `authenticate_accepted_evidence` into a
   data-returning form plus a refusing wrapper. Five callers unchanged.
2. Replace both reopen gates with the two-axis test; add `&& anchored` to the successor-coverage
   refusal.
3. Add `ReopenCauseV1` and the optional `stale_evidence_cause` field; populate it at the
   `ReopenRecord` literal.
4. Patch the sibling validator at `:1979`.
5. Add the discriminating test; prove it red on a separate checkout at `3997fc5b`.
6. Convert the existing squash-merge test from pinning the defect to pinning both directions.
7. Amend the spec invariants, requirements, Error Cases, and Public API.
8. Full suite, clippy, fmt, check.

Deliberately NOT in this change: the re-anchor half of #674, the `ignored_paths` timing fix
(#676), and the second field deadlock (approval-identity while accepted). Each is its own change.
