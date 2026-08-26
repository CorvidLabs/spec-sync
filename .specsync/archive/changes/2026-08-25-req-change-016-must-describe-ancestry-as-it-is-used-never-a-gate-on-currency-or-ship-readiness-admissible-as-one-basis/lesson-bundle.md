# Lesson bundle — req-change-016-must-describe-ancestry-as-it-is-used-never-a-gate-on-currency-or-ship-readiness-admissible-as-one-basis

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: REQ-change-016 must describe ancestry as it is used: never a gate on currency or ship readiness, admissible as one basis for archival anchoring
- **Kind**: BugFix
- **Specs**: change
- **Paths**: specs/change/requirements.md
- **Acceptance**: REQ-change-016 in specs/change/requirements.md no longer claims verification.commit is never a gate unconditionally: the claim is scoped to verification currency and ship readiness, and the requirement states that archival authentication of accepted evidence is a separate question that MAY consult commit ancestry as one basis among the integrated accepted workspace and the acceptance recorded on the remote default branch.
- **Acceptance**: The requirement carries one testable obligation the code can be measured against: ancestry MUST NOT be the only basis on which anchoring can be established.
- **Acceptance**: The materialized REQ-change-016 body in specs/change/requirements.md is byte-identical to the ### REQUIREMENT REQ-change-016 body in the semantic delta, and every other requirement in that file is untouched.
- **Acceptance**: No source behaviour changes: verification_commit_is_accepted_current keeps all three call sites, two hard conjuncts in staged_accepted_snapshot_is_closing_authenticated and one of three disjuncts in accepted_evidence_is_anchored. The two conjuncts still violate the new MUST NOT clause; that violation is tracked separately as #706 and is deliberately out of scope here.

## Evidence

- Verification commit: `39cef2d4c7b4da157776b97c54c3ac4b19d89b97`
- Base commit: `e82542d19ce8d79926b144a0e38d4d620b120715`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's testing.md

# Testing

This change edits one canonical requirement and no source, so the evidence is materialization
equality plus the unchanged suite.

- `specsync change check <id>` materializes `deltas/change.md` into `specs/change/requirements.md`
  and runs the configured commands, including `cargo test`. A green run proves no behaviour moved.
- Byte-equality assertion: the body under `### REQ-change-016` in the materialized
  `specs/change/requirements.md` is compared line-for-line (whole lines, not substrings) with the
  body under `### REQUIREMENT REQ-change-016` in `deltas/change.md`. This is asserted after
  materialization, not before.
- Neighbour check: `git diff specs/change/requirements.md` touches exactly the one bullet — two
  lines removed, six added — and no other requirement in the file.
- Delta-integrity check: `deltas/change.md` contains exactly one `## MODIFIED` block and exactly
  one `### REQUIREMENT` item, so nothing can be dropped by a regeneration that stops early. The
  file was produced by a script that compares whole lines and asserts the replaced bullet is
  present exactly once before writing.
- Not tested here: that the code obeys the new MUST NOT clause. The two conjunct sites at
  `src/change.rs:13874` and `:13879` violate it today; that is #706's discriminating test, not
  this change's.

## Where these lessons go

- `specs/change/context.md`
