---
id: the-verifying-handoff-must-not-claim-the-verification-evidence-is-committed
state: implementing
type: bug_fix
base_commit: d929a60d70903c0241d3a6411961ea168004d561
---

# The Verifying handoff must not claim the verification evidence is committed

## Intent

the Verifying handoff must not claim the verification evidence is committed

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- After a passing change check without --commit, the record is Verifying with verification.json untracked and current. The Handoff line then reads safe with a reason that says the implementation is committed and its verification is current; it no longer says the verification is committed, because HandoffSignals carries no such signal and classify_handoff never checked it. A unit test pins that no Verifying reason claims the verification is committed. The verdict, resume command, and JSON shape are unchanged.

## No-spec Rationale

Invariant 42 and REQ-change-093 prescribe the verdict and what the reason may not contain (digests); they do not prescribe the reason wording. The safe verdict in Verifying is correct — the lifecycle proceeds from uncommitted evidence — only the phrase overstates what the classifier checked.
