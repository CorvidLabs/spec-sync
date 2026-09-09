---
change: mcp-tools-lock-file-and-mcp-lock-diff
artifact: context
---

# Context

Cos-brain Research/Tools dogfood (2026-09-09) showed that MCP `tools/list` drift
is not caught the way SpecSync catches export drift. Hall decision: SpecSync owns
the lock format + CLI; consumer repos commit `.specsync/mcp-tools.lock.json`;
Fledge may wrap later as a consumer only. No mcp-sentinel / orphan scripts.

Constraints: fail-closed `diff`; never silent rewrite; one shared tool catalog with
the MCP server; omit `title` when null/absent in the canonical hash.
