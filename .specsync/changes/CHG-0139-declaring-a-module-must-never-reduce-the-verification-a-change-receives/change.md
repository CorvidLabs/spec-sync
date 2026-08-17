---
id: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
state: implementing
type: bug_fix
base_commit: 998df28e9ed932ffc78ef53f3a8481f150b6b3ed
---

# Declaring a module must never reduce the verification a change receives

## Intent

declaring a module must never reduce the verification a change receives

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- declaring an additional spec module never removes a verification command: the command set for a scope containing both a routed and an unrouted module is a superset of the set for either alone; a change scoped entirely to routed modules still runs only its component commands, so targeted verification survives; a change declaring no module at all still runs the project-wide list; the property is asserted as a superset relation rather than by example, so any future regression of this shape is caught; the new assertions fail on an unfixed binary built from a separate checkout with src/change.rs provably unmodified, and the vacuity control passes on both binaries.

## No-spec Rationale

Not applicable
