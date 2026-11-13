---
id: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now
state: draft
type: feature
base_commit: bb7682a78033aa06146baa638c44b241b6b937e1
---

# An in-flight change that declares a shared input covers an accepted change for now

## Intent

An in-flight change that declares a shared input covers an accepted change for now

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- an accepted change whose signed input an approved later change declaring its module and path has edited reads provisionally-covered naming that change; audit warns instead of failing; it archives with coverage.json naming the covering change; with no such change it is stale and audit fails as before

## No-spec Rationale

Not applicable
