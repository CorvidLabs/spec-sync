---
id: split-remaining-mcp-server-responsibilities-into-focused-private-modules
state: approved
type: refactor
base_commit: 684aed366163c29ce3b5df22b3c4716353d93fcf
---

# Split remaining MCP server responsibilities into focused private modules

## Intent

Split remaining MCP server responsibilities into focused private modules

## Affected Canonical Specs

- `mcp`

## Acceptance Criteria

- MCP server implementation and unit tests are organized into cohesive private responsibility modules; the orchestration file and each new source or test file target fewer than 1000 lines unless a documented cohesive boundary requires more; public API and CLI and MCP JSON and diagnostics and ordering and limits and filesystem authority remain unchanged; all 124 MCP unit tests and 72 integration tests remain represented and pass; canonical spec coverage stays complete and no existing test or trust gate is weakened

## No-spec Rationale

Not applicable
