---
spec: mcp.spec.md
---

## Tasks

- [ ] Add `specsync_watch` tool that streams validation results on file changes
- [ ] Add `specsync_hooks` tool for installing/checking hook status via MCP
- [ ] Add resource support (MCP resources for reading spec files directly)
- [ ] Add prompt templates for common spec-sync workflows

## Done

- [x] JSON-RPC 2.0 stdio transport
- [x] `specsync_check` tool — validate all specs
- [x] `specsync_coverage` tool — file/LOC coverage metrics
- [x] `specsync_generate` tool — create missing specs (with optional AI)
- [x] `specsync_list_specs` tool — list specs with metadata
- [x] `specsync_init` tool — create specsync.json
- [x] `specsync_score` tool — quality scoring
- [x] Error handling as `isError: true` responses
- [x] Optional `root` parameter on all tools

## Gaps

- No MCP resource support — agents can't read spec files through the protocol, only through tool results
- No streaming/progress for long-running operations (AI generation)
- No `specsync_resolve` tool for cross-project dependency resolution

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
