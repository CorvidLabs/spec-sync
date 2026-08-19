---
change: CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar
artifact: testing
---

# Testing

Every result is measured against a binary built from a **separate checkout** at `81f752c0`,
never by reverting files in the working tree.

## The gate was made able to judge before the fix existed

Sandbox PR #90 landed three assertions first, red, because two shims proved drill 053 could not
distinguish a real fix from a cosmetic one:

| shim against the unfixed tree | 053 | 030 | 031 |
|---|---|---|---|
| one-line text-printer swap | `4/0/3 → 7/0/1` — three gates flip | unchanged | unchanged |
| 3-line patch asserting `done` for archived, reading no evidence | `8/0/0` **PASS** | — | — |

## Gate 053

    unfixed  pass=5  fail=0  pending=6   verdict: FAIL
    fixed    pass=11 fail=0  pending=0   verdict: PASS

All six gates flipped, including the two that no cosmetic patch can satisfy —
`verification_commit` resolving to a 40-char sha, and `review_present` true — and including gate
4, whose two-`[current]` symptom the evidence resolution fixed as a consequence rather than by
assertion.

The corrupt-archive control stayed green in both states, which is the point of it: the unfixed
binary never reads the archive, so it can only ever catch this fix going in the wrong direction.

## Unit tests, and an honest account of which discriminate

| test | unfixed | fixed |
|---|---|---|
| `draft_ship_next_defers_to_the_lifecycle_next_action` | **FAILED** | ok |
| `archived_evidence_is_resolved_from_the_archive_package` | **FAILED** | ok |
| `a_corrupt_archived_verification_degrades_instead_of_failing` | ok | ok |
| `ship_next_is_an_action_never_a_blocker_restatement` | ok | ok |

Two discriminate. The corrupt-archive test is the deliberate vacuity control — it must pass on
both, because a strict-parse implementation is what it exists to catch.

The fourth passes on the unfixed binary too, and its doc comment now says so: a draft carries no
blockers, and the defect lives at `Approved`, which that fixture cannot reach without driving a
full interview and approval. Drill 053's approved-state gate is what actually judges that half.
Left in as a regression guard, labelled rather than dressed up.

## A wrong first attempt, caught by the gate

The blocker arm initially rendered `{action} — blocked: {blocker}`. Drill 053 rejected it,
because its gate matches the blocker text as a substring — correctly, since a `Next:` line's
contract is to be a runnable command, and a line containing a blocker restatement is not one.
The arm was deleted rather than reworded.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-change-014 | Gate 053 goes 5/0/6 FAIL to 11/0/0 PASS against a separate-checkout binary, with the two evidence gates and the corrupt-archive control landed before the fix so the board could not be satisfied by a cosmetic patch. Two unit tests fail on unfixed source and pass on fixed; the corrupt-archive control passes on both by design. Enumeration found exactly two production sites constructing the active workspace path by hand (`commands/change.rs:772`, `:803`); the other two are inside `#[cfg(test)]`. The fix routes both through the existing `find_change_dir`, removing two parallel implementations of `change_dir` rather than adding a third idiom |
