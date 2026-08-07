## ADDED

### REQUIREMENT REQ-cmd-change-007

`specsync change ship-status` SHALL report local ship readiness for an active change
without querying GitHub check-runs: verification tip presence and ancestry relative
to HEAD, whether a scoped review is recorded, blockers, warnings (including
merge-before-finalize), and a concrete next action.

Acceptance Criteria

- JSON includes `verification_present`, `verification_ancestor_of_head`,
  `review_present`, `ready_to_finalize`, `blockers`, `warnings`, and `ship_next`.
- An absent or non-ancestor verification commit is a blocker naming re-check.
- A verifying change always warns not to merge before finalize.
