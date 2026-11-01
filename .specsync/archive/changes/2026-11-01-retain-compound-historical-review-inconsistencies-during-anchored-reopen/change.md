---
id: retain-compound-historical-review-inconsistencies-during-anchored-reopen
state: archived
type: bug_fix
base_commit: 9eba6ed1a9f24226fe24f08d890bbbd70245dbb5
---

# Retain compound historical review inconsistencies during anchored reopen

## Intent

Retain compound historical review inconsistencies during anchored reopen

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Within exact recorded workspace-mismatch recovery, preserve the entire historical review including its inconsistent claimed hash as untrusted; keep every normal validation and non-review digest strict, refuse a pure review-hash mismatch, require exact remote-default package anchoring and fresh verification/review, and qualify the public source against the tested implementation.

## No-spec Rationale

Not applicable
