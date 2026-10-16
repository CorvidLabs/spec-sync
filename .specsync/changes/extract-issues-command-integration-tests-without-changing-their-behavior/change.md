---
id: extract-issues-command-integration-tests-without-changing-their-behavior
state: draft
type: refactor
base_commit: 06d343e630deb7ad14eca93b242f7060b0928646
---

# Extract issues command integration tests without changing their behavior

## Intent

Extract issues command integration tests without changing their behavior

## Affected Canonical Specs

- None

## Acceptance Criteria

- Agent-proposed outcomes: conserve all26 test bodies/attributes/fixtures and shared helper behavior, select identical24macOS tests before/after, pass full native2582unit/462integration, retain unexecuted platform limits, update documented filters and preserve product behavior.

## No-spec Rationale

Only byte-conserved integration tests and their testing-companion paths change; product API, source behavior and canonical spec contract remain unchanged. Draft records the required CI coverage without human approval.
