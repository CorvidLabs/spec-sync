---
id: split-mcp-issue-verification-into-focused-private-modules
state: implementing
type: refactor
base_commit: 684aed366163c29ce3b5df22b3c4716353d93fcf
---

# Split MCP issue verification into focused private modules

## Intent

Split MCP issue verification into focused private modules

## Affected Canonical Specs

- `mcp`

## Acceptance Criteria

- MCP issue verification and its existing regression tests live in cohesive private files smaller than 1000 lines; public API and JSON output and diagnostics and security limits remain unchanged; all existing MCP tests remain represented and pass; canonical spec file coverage remains complete

## No-spec Rationale

Not applicable
