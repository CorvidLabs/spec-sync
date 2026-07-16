---
id: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
state: verifying
type: bug_fix
base_commit: 2646fc3495bb4f5125e6f4f463f59e58f9f93110
---

# Harden canonical numeric change ordering across CHG-9999 to CHG-10000 and correct 5.1 release documentation

## Intent

Harden canonical numeric change ordering across CHG-9999 to CHG-10000 and correct 5.1 release documentation

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- CHG-10000 sorts after CHG-9999 by numeric sequence.
- Same-sequence canonical IDs use the full ID as a deterministic tie-breaker.
- Malformed, noncanonical, and overflowing IDs fail closed.
- Regression coverage exercises the numeric boundary and invalid forms.
- The 5.1 changelog names extensionless export-star resolution to sibling .mjs and .cjs.
- Both adversarial-proof matrices identify SpecSync 5.1.
- The Trust workflow describes its immutable SHA as an unreleased candidate and does not claim a nonexistent v1.0.1 tag.

## No-spec Rationale

Not applicable
