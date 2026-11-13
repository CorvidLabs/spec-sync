---
change: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
artifact: delta
module: cmd_change
---

## ADDED

### REQUIREMENT REQ-cmd-change-038

The documented way for CI to check one archived change SHALL be
`specsync change status <id> --json`, asserting that `change.state` is `archived` and that
`summary.terminal_evidence.validity` is `authenticated-history`. The page SHALL say what the read
re-derives from Git, what it trusts, what it remembers and what it ignores, and that
`--timeout <secs>` ends an over-long read with exit 124. A test SHALL run the check the page
prints, so the page cannot drift from the output (#1003).

#### Acceptance Criteria

- The check printed under "Check one archived change" passes on an intact archived change and
  fails once the archived evidence is edited and committed.
- The page names both fields, the validity values that fail the check, the trust model and
  `--timeout` with exit 124.
- The test reads the check from the page rather than restating it.
