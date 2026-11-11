---
id: indented-headers-end-requirement-blocks
state: implementing
type: bug_fix
base_commit: e04fb1e36d44197efa32c229322b28a925a7c5ea
---

# Indented headers end requirement blocks

## Intent

indented headers end requirement blocks

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a MODIFIED or REMOVED on an indented titled header leaves the following indented requirement block intact

## No-spec Rationale

round 2's matcher fix let indented headers match but the block-end scan still required column 0; this restores the contract REQ-change-150 already states
