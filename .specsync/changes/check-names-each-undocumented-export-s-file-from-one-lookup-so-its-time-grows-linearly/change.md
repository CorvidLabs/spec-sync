---
id: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Check names each undocumented export's file from one lookup, so its time grows linearly

## Intent

check names each undocumented export's file from one lookup, so its time grows linearly

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- with four times as many undocumented exports, check takes well under seven times as long, and each warning names the same file as before

## No-spec Rationale

SPEED-1.b: check stays proportional to the specs it touches; naming each undocumented export's file rescanned every export, so check time grew with the square of their number (#574). Output is unchanged
