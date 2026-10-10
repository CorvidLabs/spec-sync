---
change: split-remaining-mcp-server-responsibilities-into-focused-private-modules
artifact: context
---

# Context

The user approved publishing the first issue-verification split as a draft PR and explicitly asked to continue on SpecSync. The first snapshot is frozen separately. This continuation starts with that patch applied in its own worktree and declares a dependency on its change; it will be reconciled onto the exact first published commit when available.

This is one batch scope for the remaining MCP server implementation and unit tests, not a separate approval for every source file. The issue module already measures 272 lines and its tests 400; src/mcp.rs still measures 8572. The requested outcome is maintainable modules, with files below 1000 lines where cohesive boundaries allow it. No public behavior or security contract changes are intended.

The first split passed all 124 MCP unit and 72 integration tests, formatter, compiler, clippy, release build, strict 100% source coverage, tool-lock comparison and pre-push gate. Full before/after suites had identical 184 baseline failures, primarily sandbox ancestor Git markers. Ruby/Psych and fledge were unavailable. These limits are not permissions to weaken any gate.

## Local Checkpoint and Publication Status

The first split is committed at e1d65d79bed4aa0bdc390a7901e6a60b4bf41d1f using the existing verified Corvid Agent Git author identity. This continuation is stacked locally on that exact tree. The user explicitly requested a branch and commits and instructed terminal git/gh publication only. No browser publication is authorized for this continuation. The remote first-split branch is incomplete and must be reconciled safely through terminal Git once authentication is available; no force-push or merge is part of this change.
