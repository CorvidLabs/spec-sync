---
id: CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete
state: archived
type: bug_fix
base_commit: 07a044a3e33987630ee8d53c000e71ca89074962
---

# Fail lifecycle verification before running the suite when evidence is incomplete, make already-applied ADDED deltas converge, and reject duplicate change ordinals from one base

## Intent

Fail lifecycle verification before running the suite when evidence is incomplete, make already-applied ADDED deltas converge, and reject duplicate change ordinals from one base

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Verification refuses to start when acceptance or requirement evidence is incomplete, naming the change testing.md and its Requirement evidence table rather than verification.json; command failures name the failing command and exit code; an ADDED delta whose block is already present with byte-identical content converges instead of erroring, while present-but-different still errors and directs the author to MODIFIED; two distinct changes claiming the same CHG-NNNN ordinal from the same base commit are rejected at approve and in change audit, while differing or unknown base commits never raise a collision

## No-spec Rationale

Not applicable
