## MODIFIED

### REQUIREMENT REQ-cmd-change-007

`specsync change ship-status` SHALL report local ship readiness for an active
change without requiring GitHub check-run queries:

- HEAD tip class: `product`, `review_only`, `archive_only`, or `other`
- verification tip presence and ancestry relative to HEAD
- whether a scoped review is recorded
- trust guidance for staged product → review → archive tips
- ordered ship stages with concrete next actions
- blockers, warnings (including merge-before-finalize), and `ship_next`

Acceptance Criteria

- JSON includes `tip_class`, `tip_sha`, `parent_sha`, `trust`, `stages`,
  `verification_present`, `verification_ancestor_of_head`, `review_present`,
  `ready_to_finalize`, `blockers`, `warnings`, and `ship_next`.
- Tip class is derived from the paths changed in HEAD relative to its first parent
  (or working-tree vs HEAD when HEAD is not a useful tip).
- An absent or non-ancestor verification commit is a blocker naming re-check.
- A verifying change always warns not to merge before finalize.

## ADDED

### REQUIREMENT REQ-cmd-change-008

`specsync change ship [ID]` SHALL run ship preflight for one change and, when
`ready_to_finalize` is true, perform finalize. When not ready it SHALL exit
non-zero and print blockers and the next stage without mutating state.

Acceptance Criteria

- Exit code 0 only when preflight is clean and finalize succeeds (or the change
  is already archived and nothing remains).
- Exit code non-zero when blockers remain.
- Text and JSON outputs name the current tip class and next ship stage.
