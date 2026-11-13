---
id: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# A requirement whose SHALL sits at a line edge counts as a normative statement

## Intent

A requirement whose SHALL sits at a line edge counts as a normative statement

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a requirement whose SHALL ends or starts a wrapped line is accepted; a requirement with no SHALL is still refused with the normative SHALL message

## No-spec Rationale

A requirement must contain a normative SHALL statement; a SHALL that a line break sets off, at the end or start of a wrapped line, is that statement, and validate_requirement refused it because it looked only for a space on each side (#1076)
