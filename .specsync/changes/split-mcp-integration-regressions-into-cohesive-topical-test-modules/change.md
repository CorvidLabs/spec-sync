---
id: split-mcp-integration-regressions-into-cohesive-topical-test-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split MCP integration regressions into cohesive topical test modules

## Intent

Split MCP integration regressions into cohesive topical test modules

## Affected Canonical Specs

- `mcp`

## Acceptance Criteria

- Reduce the 4139-line MCP integration test file to a small registered facade and cohesive private topical modules below 1000 lines; preserve all 77 test declarations, 72 Linux cases, complete test and helper bodies, fixture literals, assertions, platform attributes, four release-ignore guards, FIFO and socket PermissionDenied semantics, Windows command-helper seam, and root integration registration; change no production code or public behavior and document the test topology with faithful lifecycle evidence

## No-spec Rationale

Not applicable
