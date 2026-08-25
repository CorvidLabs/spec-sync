---
change: req-change-016-must-describe-ancestry-as-it-is-used-never-a-gate-on-currency-or-ship-readiness-admissible-as-one-basis
artifact: context
---

# Context

Issue #698 was found during the #694 design pass by reading REQ-change-016 and `src/change.rs`
side by side. The requirement says:

> `verification.commit` is retained as an informational correlation key and is never a gate; a
> squash merge that discards the recorded commit does not invalidate the evidence.

`verification_commit_is_accepted_current` is `git merge-base --is-ancestor` and nothing else, and
it is consulted at three sites:

- `src/change.rs:13874` — in `staged_accepted_snapshot_is_closing_authenticated`, workflow v2 branch:
  `validate_finalization_evidence(root, accepted, &verification).is_ok() && verification_commit_is_accepted_current(root, &verification)`
- `src/change.rs:13879` — in the same function, legacy branch:
  `Ok(closing_matches && verification_commit_is_accepted_current(root, &verification))`
- `src/change.rs:14608` — in `accepted_evidence_is_anchored`:
  `verification_commit_is_accepted_current(root, evidence) || accepted_workspace_is_integrated(root, record) || accepted_change_is_recorded_on_remote_default(root, record)`

So the sentence is false as written. Which side is wrong was the real question, and it was decided
before this change was opened: the requirement overclaims, the code is right — but only for
archival authentication.

#689 already removed ancestry from ship readiness, where it was freshness wearing a trust costume.
What remains is archival authentication over history-discovered commits, where ancestry is
load-bearing trust: it answers "is this acceptance anchored in history a reader can reach", which
is a different question from "is this evidence current". `never a gate` was written to describe the
readiness path and then stated unconditionally.

## Why the obvious narrowing is also wrong

The tempting wording — "ancestry may be consulted as one basis among several" — would be FALSE at
two of the three sites. `:14608` is one disjunct of three, so ancestry there can only widen
acceptance. `:13874` and `:13879` are hard conjuncts inside
`staged_accepted_snapshot_is_closing_authenticated`; there ancestry can only block, and it is the
sole basis. A requirement that describes all three as "one basis among several" would be a second
false sentence replacing the first.

The wording landed here therefore does two things at once: it describes the disjunct site
truthfully, and it states one obligation — *ancestry MUST NOT be the only basis on which anchoring
can be established* — that the two conjunct sites currently violate. That is deliberate. It is the
testable clause, and the violation is tracked separately as #706.

## Ruled out

- Deleting the three call sites to match the sentence. The issue explicitly warns against it: it
  would be a fix landing where the report points, and it would remove real archival trust.
- Changing any source file. This change is the requirement catching up to reality plus one stated
  obligation; no behaviour moves here.
- Restating the whole requirement. Only the second acceptance-criteria bullet changes; the other
  five are reproduced byte-for-byte so the delta cannot silently rewrite neighbours.
