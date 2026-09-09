---
id: cover-mcp-tools-lock-file-and-mcp-lock-diff-delivery-scope-for-src-main-rs-mcp-lock-diff-dispatch-wiring-omitted-from
state: implementing
type: bug_fix
base_commit: 9d20c4c6f3f51534732cdf798315999bf3374126
---

# Cover mcp-tools-lock-file-and-mcp-lock-diff delivery scope for src/main.rs MCP lock/diff dispatch wiring omitted from the parent change affected_paths

## Intent

Cover mcp-tools-lock-file-and-mcp-lock-diff delivery scope for src/main.rs MCP lock/diff dispatch wiring omitted from the parent change affected_paths

## Affected Canonical Specs

- `cli`

## Acceptance Criteria

- src/main.rs is covered by an active change in delivery-diff coverage; specsync change audit --strict reports no uncovered meaningful changed paths for this path; no canonical spec content changes

## No-spec Rationale

Bookkeeping delivery-scope coverage: MCP lock/diff CLI dispatch in src/main.rs landed under accepted mcp-tools-lock-file-and-mcp-lock-diff but was not declared in its affected_paths; this change only declares ownership so delivery-diff coverage is complete
