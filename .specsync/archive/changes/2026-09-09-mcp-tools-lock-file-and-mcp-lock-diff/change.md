---
id: mcp-tools-lock-file-and-mcp-lock-diff
state: archived
type: feature
base_commit: 7fc6912d2e0afd12f4e97a1a70c77f961aba8a10
---

# MCP tools lock file and mcp lock/diff

## Intent

MCP tools lock file and mcp lock/diff

## Affected Canonical Specs

- `mcp`
- `cli_args`

## Acceptance Criteria

- specsync mcp lock prints lock JSON; mcp lock --write writes .specsync/mcp-tools.lock.json; mcp diff exits 0 on match and non-zero on add/remove/rename/sha256/tool_count/missing-lock drift with remedy; unit tests cover match, description/schema drift, missing lock; dogfood lock committed; specs/docs/CHANGELOG updated

## No-spec Rationale

Not applicable
