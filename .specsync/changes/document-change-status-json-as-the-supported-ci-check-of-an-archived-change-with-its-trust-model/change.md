---
id: document-change-status-json-as-the-supported-ci-check-of-an-archived-change-with-its-trust-model
state: implementing
type: documentation
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Document change status --json as the supported CI check of an archived change, with its trust model

## Intent

Document change status --json as the supported CI check of an archived change, with its trust model

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- the CI archive check page's own bash block passes on an intact archived change and fails once its archived evidence is edited and committed; the page names change.state, summary.terminal_evidence.validity, the trust model and --timeout with exit 124; the Action page links to it

## No-spec Rationale

Not applicable
