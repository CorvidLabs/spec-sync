---
id: require-explicit-succession-after-temporary-coverage
state: draft
type: bug_fix
base_commit: 6e5a63b3d9d83151dde78734b4557a577d1a494b
---

# Require explicit succession after temporary coverage

## Intent

Require explicit succession after temporary coverage

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- After a temporary coverer is accepted, an active predecessor gets successor coverage only through an explicit valid succession obligation. On an archived predecessor, status, show and archive audit continue to warn for a file when the accepted or archived coverer lacks that obligation; a valid matching succession clears the warning and archive integrity remains independent.

## No-spec Rationale

Not applicable
