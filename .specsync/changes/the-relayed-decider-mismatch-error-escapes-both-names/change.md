---
id: the-relayed-decider-mismatch-error-escapes-both-names
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The relayed decider-mismatch error escapes both names

## Intent

The relayed decider-mismatch error escapes both names

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- The decider-mismatch error, on write (change approve or review with a relay) and on read (change status, show and every ledger read), shows the relayed decided_by and the record's actor or reviewer with control characters escaped, so a stored escape sequence or newline is printed as its escape (red then green)

## No-spec Rationale

Makes the decider-mismatch error match REQ-cmd-change-relayed-decision-001, whose text output escapes control characters in recorded names and relayed fields; the error printed both names raw.
