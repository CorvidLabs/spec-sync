---
spec: mcp.spec.md
---

## Tasks

- [ ] Add `specsync_watch` tool that streams validation results on file changes
- [ ] Add `specsync_hooks` tool for installing/checking hook status via MCP
- [ ] Add prompt templates for common spec-sync workflows
- [ ] Add `specsync_resolve` tool for cross-project dependency resolution

## Done

- [x] JSON-RPC 2.0 stdio transport
- [x] `specsync_check` tool — validate all specs (with staleness warnings)
- [x] `specsync_coverage` tool — file/LOC coverage metrics
- [x] `specsync_generate` tool — create missing specs (optional AI via `ai::resolve_ai_provider`)
- [x] `specsync_list_specs` tool — list specs with metadata
- [x] `specsync_init` tool — create specsync.json
- [x] `specsync_score` tool — quality scoring
- [x] `specsync_issues` tool — verify GitHub issue references in frontmatter
- [x] MCP resources: `specsync:///specs`, `:///graph`, `:///config`, `:///coverage` + `:///specs/{module}` template
- [x] Error handling as `isError: true` for tools, JSON-RPC -32602 for resource reads
- [x] Optional `root` parameter on all tools

## Gaps

- No streaming/progress for long-running operations (AI generation)
- No MCP prompts (tools and resources only)
- No `specsync_resolve` tool for cross-project dependency resolution

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
