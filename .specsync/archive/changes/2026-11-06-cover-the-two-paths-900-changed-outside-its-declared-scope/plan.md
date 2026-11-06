---
change: cover-the-two-paths-900-changed-outside-its-declared-scope
artifact: plan
---

# Plan

## Approach

Open a no-spec-change change whose affected paths are the two files #900 changed outside its declared scope, and take it through approval, check, review and ship on #900, so that `change audit --strict` passes on the archive tip.

## Out of scope

- Any edit to either file, or to any other file the pull request already covers.
- The deferred Codex P2s (#926, #927) and #904.

## Steps

1. Create this change with `--no-spec-change` and the two paths; acceptance criteria CHANGE-6.d and CHANGE-13.b.
2. After approval: `change check --commit`, `specsync change audit --strict`, pre-push and the verify lane.
3. After review: `change ship` with no commit in between, the audit again, push, and CI.

## Risks

- Any further file edited on this pull request must already be covered. Running `change audit --strict` before every push checks that.

## Constraints consulted

AGENTS.md "a fix that changes the approved scope needs a new change"; the audit's own remediation; never weaken a gate, bound or test.
