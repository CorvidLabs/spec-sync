# Lesson bundle — mcp-tools-lock-file-and-mcp-lock-diff

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: MCP tools lock file and mcp lock/diff
- **Kind**: Feature
- **Specs**: mcp, cli_args
- **Paths**: src/mcp.rs, src/mcp_tools_lock.rs, src/cli.rs, .specsync/mcp-tools.lock.json, specs/mcp/mcp.spec.md, specs/cli_args/cli_args.spec.md, CHANGELOG.md, site/src/content/docs/cli.md, site/src/content/docs/workflow.md, site/src/content/docs/integrations/ai-agents.md
- **Acceptance**: specsync mcp lock prints lock JSON; mcp lock --write writes .specsync/mcp-tools.lock.json; mcp diff exits 0 on match and non-zero on add/remove/rename/sha256/tool_count/missing-lock drift with remedy; unit tests cover match, description/schema drift, missing lock; dogfood lock committed; specs/docs/CHANGELOG updated

## Evidence

- Verification commit: `e61d7e1a9d716a530a0512686d44829e16de8f2b`
- Base commit: `7fc6912d2e0afd12f4e97a1a70c77f961aba8a10`
- Verified by: `specsync check --spec cli_args --spec mcp`

## From the change's context.md

# Context

Cos-brain Research/Tools dogfood (2026-09-09) showed that MCP `tools/list` drift
is not caught the way SpecSync catches export drift. Hall decision: SpecSync owns
the lock format + CLI; consumer repos commit `.specsync/mcp-tools.lock.json`;
Fledge may wrap later as a consumer only. No mcp-sentinel / orphan scripts.

Constraints: fail-closed `diff`; never silent rewrite; one shared tool catalog with
the MCP server; omit `title` when null/absent in the canonical hash.

## From the change's design.md

# Design

- Extract `mcp_tool_definitions(allow_write)` from `handle_tools_list` so server and
  lock share one catalog.
- New module `mcp_tools_lock` builds lock entries, hashes controlled fields, writes
  the lock, and diffs live vs committed.
- CLI: optional `McpAction` under `Command::Mcp` — `Lock { write, allow_write }` and
  `Diff { allow_write }`; bare `specsync mcp` still starts the server.
- Diff never writes. Missing lock is fail-closed with remedy `mcp lock --write`.

## From the change's testing.md

# Testing

- Unit: `cargo test mcp_tools_lock` — match, description drift, schema drift,
  missing lock, removed tool, catalog identity, title omission.
- Smoke: `specsync mcp lock --write` then `specsync mcp diff` → exit 0; missing
  root → exit 1; corrupted sha → exit 1.
- Spec: `specsync check --strict mcp cli_args` → pass.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-mcp-009 | `match_returns_ok`, `missing_lock_returns_err_with_remedy`, `description_drift_returns_nonzero`, `schema_drift_via_sha_mismatch`, `removed_tool_detected`, `lock_uses_same_catalog_as_server`, `sha256_omits_null_title`, `canonical_json_sorts_keys_and_is_compact` |
| REQ-cli-args-018 | `lock_uses_same_catalog_as_server`, `match_returns_ok`, `missing_lock_returns_err_with_remedy` |

## Where these lessons go

- `specs/mcp/context.md`
- `specs/cli_args/context.md`
