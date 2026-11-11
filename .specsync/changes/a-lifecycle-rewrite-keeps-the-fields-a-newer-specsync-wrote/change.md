---
id: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
state: draft
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# A lifecycle rewrite keeps the fields a newer specsync wrote

## Intent

a lifecycle rewrite keeps the fields a newer specsync wrote

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A lifecycle record or ledger SpecSync rewrites in place keeps every member it does not know, with its exact text, at the top level and inside entries the rewrite left unchanged, through send-back, answer, approve, check, review, ship and relay repair, and the audit stays clean

## No-spec Rationale

Not applicable
