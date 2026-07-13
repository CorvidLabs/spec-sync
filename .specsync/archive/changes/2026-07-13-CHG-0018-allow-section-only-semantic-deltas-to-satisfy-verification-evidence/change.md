---
id: CHG-0018-allow-section-only-semantic-deltas-to-satisfy-verification-evidence
state: archived
type: bug_fix
base_commit: 52b47f1bb9c7434d7bbcac7b2ac8a7c477737cd6
---

# Allow section-only semantic deltas to satisfy verification evidence

## Intent

Allow section-only semantic deltas to satisfy verification evidence

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A section-only non-removed semantic delta verifies with empty requirement IDs; requirement mappings remain required for requirement deltas; missing semantic evidence is reported separately from command failure

## No-spec Rationale

Not applicable
