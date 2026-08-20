## ADDED

### REQUIREMENT REQ-change-081

A gate SHALL determine a change's identity from its persisted state rather than from the shape of a directory or file name, and a gate that cannot determine identity SHALL withhold the permission it grants rather than granting it.

Acceptance Criteria
- An archived package that has lost its lifecycle state is refused as damaged whatever it is named, because a naming convention is not evidence that a package is real and skipping a damaged package hides corruption.
- A genuine pre-lifecycle record, holding deltas and nothing else, continues to be skipped, so refusing damage is not achieved by refusing everything.
- Continuous integration determines which changes require an independent review by reading persisted state, so no identity shape can reduce the set of changes needing review to zero and let a pull request merge unreviewed while reporting success.
- A gate that cannot read identity withholds what it grants: an archive fast lane is not taken when the archived state is unreadable, so the full verification runs instead.
