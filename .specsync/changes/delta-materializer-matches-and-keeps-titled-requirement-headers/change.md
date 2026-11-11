---
id: delta-materializer-matches-and-keeps-titled-requirement-headers
state: draft
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Delta materializer matches and keeps titled requirement headers

## Intent

delta materializer matches and keeps titled requirement headers

## Affected Canonical Specs

- `change`
- `validator`

## Acceptance Criteria

- a MODIFIED delta applies to every header form the requirement parser accepts (title suffix, punctuation, multi-segment ids, up to 3-space indent); the matched header line is written back unchanged; a following indented requirement block survives

## No-spec Rationale

Not applicable
