---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: design
---

# Design

One resolver, two readers. There is exactly one implementation of "is this evidence anchored",
and it is asked as a question by `reopen` and enforced as a refusal by everyone else.

## Shape

`authenticate_accepted_evidence` becomes a thin wrapper over
`authenticate_accepted_evidence_with_anchor`, which returns the anchor as data:

    fn accepted_evidence_is_anchored(root, record, evidence) -> bool {
        verification_commit_is_accepted_current(root, evidence)
            || accepted_workspace_is_integrated(root, record)
            || accepted_change_is_recorded_on_remote_default(root, record)
    }

All five existing callers keep byte-identical behaviour through the wrapper. Reopen calls the
`_with_anchor` form and reads the third disjunct as a fact rather than a refusal.

## The gate becomes two axes, not one

    let inputs_drifted = current != stale;
    let anchored = ...;
    if !inputs_drifted && anchored { refuse }

`&& anchored` also guards the successor-coverage refusal. Successor coverage is a *content*
argument and cannot rescue an orphaned commit, so refusing there would preserve the dead end.

## The cause is recorded, because a sibling reads digest equality as proof

`reopened_change_preserves_sequence_history` (`src/change.rs:1979`) independently encodes
"a reopen implies the digests differ". An anchor-axis reopen has EQUAL digests, so without a
recorded cause that validator would strip `historical` status and — for a member of an
acknowledged sequence collision — freeze `change new` project-wide.

So `ReopenRecord` gains:

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_evidence_cause: Option<ReopenCauseV1>,

`skip_serializing_if` keeps every existing `approvals.json` byte-identical, matching invariant 17
and the additive-field precedent at `src/change.rs:734`. The sibling then admits equal digests
only when the cause is present.

## Scope of the sibling exposure, measured

Narrower than first framed: `reopened_change_preserves_sequence_history` returns false for any
`from_state != Accepted`, and 6.0's `finalize` accepts and archives atomically. So the freeze
bites only workflow-v1 Accepted-origin reopens. Patching it is still correct.
