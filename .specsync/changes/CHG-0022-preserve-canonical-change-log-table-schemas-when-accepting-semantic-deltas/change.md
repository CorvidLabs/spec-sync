---
id: CHG-0022-preserve-canonical-change-log-table-schemas-when-accepting-semantic-deltas
state: accepted
type: bug_fix
base_commit: a36af58a4f9e79d5076059d050d6f41a0f14529d
---

# Preserve canonical Change Log table schemas when accepting semantic deltas

## Intent

Preserve canonical Change Log table schemas when accepting semantic deltas

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Accepted changes append rows matching the existing Change Log column order and count; versioned tables use the bumped canonical version; author tables identify SpecSync; legacy two-column tables remain unchanged; regression tests and strict verification pass

## No-spec Rationale

Not applicable
