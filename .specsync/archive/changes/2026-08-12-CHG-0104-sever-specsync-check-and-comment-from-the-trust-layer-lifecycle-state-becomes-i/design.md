---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: design
---

# Design

## Chosen shape

Replace the `audit_project` gate in `src/commands/check.rs` with an
informational-only lifecycle summary:

- one line naming the count of active changes,
- shape warnings to stderr for unparseable / illegally-stated workspace files,
- **no** contribution to the exit code, in any format mode.

`specsync comment` (`src/commands/comment.rs`) stops merging
`check_project_quiet`'s SDD errors and warnings into its reported totals.

## Why informational rather than dropped entirely

Dropping the section outright would silently remove a signal people currently rely
on to notice they have work in flight. An informational line preserves the signal
and removes the authority. Shape warnings are retained because a corrupt workspace
file is a *content* problem the user can act on, not a trust judgement.

## Why the exit-code hole is acceptable here

Severing the gate leaves `check` unable to fail by default, because default
enforcement is `warn`. That hole is real but deliberately not closed in this
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
means the two user-visible exit-code changes can be bisected independently, which
matters because both alter CI behaviour for every consumer.

## Rejected

- **Keep a reduced trust check that still gates.** Rejected: any gate here
  re-creates the coupling, and the failures it would gate on are precisely the ones
  moving to attest.
- **Flip the enforcement default in this change.** Rejected: two exit-code changes
  in one commit are indistinguishable under bisect.
