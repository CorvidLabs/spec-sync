---
id: CHG-0032-address-all-actionable-review-findings-on-pr-370-with-regression-coverage
state: archived
type: bug_fix
base_commit: f622d09778e0e78b1fd8dea97f7c5e657e2b7c79
---

# Address all actionable review findings on PR 370 with regression coverage

## Intent

Address all actionable review findings on PR 370 with regression coverage

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Historical sequence reconstruction filters later collisions; direct and cargo-selected SpecSync verification commands are rejected before execution; both registry authority files require lifecycle coverage; affected specs cover only their exact canonical spec and requirements companion; Cargo package parsing handles spaced headers
- literal strings
- and trailing comments; all regressions and the full native suite pass.

## No-spec Rationale

Not applicable
