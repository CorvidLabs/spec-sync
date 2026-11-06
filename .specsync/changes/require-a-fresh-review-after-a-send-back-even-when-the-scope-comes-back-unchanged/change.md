---
id: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
state: implementing
type: bug_fix
base_commit: 8a251d7a553f5c98dc1f2f505262f4b0d055e3f7
---

# Require a fresh review after a send-back even when the scope comes back unchanged

## Intent

Require a fresh review after a send-back even when the scope comes back unchanged

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A change reviewed, sent back and re-approved with its scope unchanged, then checked without --commit, is not ready to ship: change status names change review, change ship-status reports the review stale, and change ship and change finalize refuse until a fresh review is recorded; the fresh review then finalizes the change with both review attempts in its history, the first unchanged.
The integration test built from the #960 scenario (the Codex P1 on #944) fails on main 8a251d7a and passes with the fix.
A send-back records how many review attempts preceded it, beside cancelled_approvals, and a review counts only when it was recorded after the latest send-back; a send-back recorded before that field existed falls back to a strict timestamp comparison, so no existing archive changes its verdict.
Review history validity (#914), the CHANGE-12.b currency rule and every other review currency check are unchanged; no gate, bound or timeout is relaxed.

## No-spec Rationale

Not applicable
