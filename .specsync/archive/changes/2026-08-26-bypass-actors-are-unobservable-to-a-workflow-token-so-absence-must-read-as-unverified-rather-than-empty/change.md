---
id: bypass-actors-are-unobservable-to-a-workflow-token-so-absence-must-read-as-unverified-rather-than-empty
state: archived
type: bug_fix
base_commit: d508f144a1d965b395abfe45f23c8b4e8978cd5f
---

# Bypass actors are unobservable to a workflow token, so absence must read as unverified rather than empty

## Intent

Bypass actors are unobservable to a workflow token, so absence must read as unverified rather than empty

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- an absent bypass_actors field validates as unverified and names what was not checked, rather than failing the gate
- a visible bypass actor is still refused
- an admin payload with no bypass actors passes with no notice

## No-spec Rationale

Not applicable
