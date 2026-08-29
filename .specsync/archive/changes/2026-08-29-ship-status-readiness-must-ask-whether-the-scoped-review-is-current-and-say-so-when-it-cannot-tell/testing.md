---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: testing
---

# Testing

All three new assertions were run against a binary built from a SEPARATE checkout of unfixed `main`
(a fresh clone at `7df4077`), never by reverting in place.

## DISCRIMINATOR — `ship_status_and_finalize_agree_when_the_review_is_stale_by_content`

- `REQ-cmd-change-007`: readiness and finalization reach the same conclusion.
- Records a review, changes the implementation, then re-runs `check` so that the VERIFICATION half
  is healthy again — without that isolation the verification blocker alone would make readiness
  false and the test would prove nothing, because every content digest the review checks is also
  checked by verification.
- Asserts AGREEMENT (`ready_to_finalize == finalize.is_ok()`), not `ready_to_finalize == false`.
  All three of #694's options end with the two agreeing, so the narrower assertion would go red the
  day #694 is resolved the other way.
- Verbatim failure on the control binary:

```
assertion `left == right` failed: ship-status and finalize disagree about the same change in the
same second: ready_to_finalize=true,
finalize=Err("independent scoped review is stale; open or update the PR so `SpecSync scoped review`
can run"), report={... "ready_to_finalize":true, "review_present":true, "blockers":[] ...}
  left: true
```

## DISCRIMINATOR — `ship_status_and_finalize_agree_after_a_squash_that_preserves_content`

- `REQ-cmd-change-007`, `REQ-change-046`: an unavailable guarantee is never reported as satisfied.
- The #689 test, relabelled. Its verification half is asserted unchanged (no verification blocker
  while `verification_ancestor_of_head` is false). Its `ready_to_finalize: true` assertion was the
  defect written down as an expectation, and is now an agreement assertion.
- Verbatim failure on the control binary: identical shape, `ready_to_finalize=true` beside
  `finalize=Err("independent scoped review is stale; ...")`.

## CONTROL — `ship_status_is_ready_when_the_scoped_review_is_current`

- Honest label: this PASSES on the unfixed binary, and that is the point. Without it, "make
  readiness stricter" clears both discriminators while breaking every healthy change.
- Confirmed passing against the control binary in the same run that failed the two above
  (`1 passed; 2 failed`).

## End to end

Sandbox drill 008's declared PENDING GATE (#694) now clears against the release binary:

```
OK: squash-merge before finalize strands the change and fails closed
OK: ship-status still names the recovery (product_tip: change check --commit)
OK: ship-status and finalize agree (finalize rc=1, ready_to_finalize=0)
OK: re-checking on the squashed tip recovers the change and finalize succeeds
DRILL 008 PASSED
```

The last line matters as much as the gate: the recovery is still reachable, so this is not a
permanent red.
