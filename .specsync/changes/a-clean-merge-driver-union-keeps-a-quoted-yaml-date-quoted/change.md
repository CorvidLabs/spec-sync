---
id: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# A clean merge-driver union keeps a quoted YAML date quoted

## Intent

A clean merge-driver union keeps a quoted YAML date quoted

## Affected Canonical Specs

- `merge`

## Acceptance Criteria

- A quoted YYYY-MM-DD list item stays a quoted string after a clean union of files in either merge order; a full timestamp stays quoted; a path holding a date stays plain (red then green)

## No-spec Rationale

Makes the merge driver match REQ-merge-006: a clean merge keeps what both sides meant, so a list item YAML 1.1 would read as a timestamp when plain stays a quoted string, as numbers, booleans and nulls already do.
