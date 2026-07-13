---
id: CHG-0020-harden-reopened-acceptance-compatibility-and-canonical-governance
state: archived
type: bug_fix
base_commit: eca4a64cf91d5f263caa542077e6639976f13cd6
---

# Harden reopened acceptance compatibility and canonical governance

## Intent

Harden reopened acceptance compatibility and canonical governance

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Reopened reacceptance accepts compatible omitted or transitional explicit-false lifecycle definition encodings
- No-spec accepted changes never satisfy canonical successor governance
- Reopened canonical-applied verifying changes validate current canonical modules without reapplying their deltas
- Focused regressions and the repo-pinned native test, release, and audit stages pass; aggregate strict SpecSync and Trust remain configured for the final stale-evidence refresh

## No-spec Rationale

Not applicable
