---
id: a-sent-back-no-spec-change-can-declare-that-it-changes-specs
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# A sent-back no-spec change can declare that it changes specs

## Intent

A sent-back no-spec change can declare that it changes specs

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a no-spec change, approved and then sent back, answers no_spec_change no, adds a delta and is approved, materialized and verified with the delta applied; no_spec_change yes, no on a change that already changes specs, and any answer on an approved change are refused

## No-spec Rationale

Not applicable
