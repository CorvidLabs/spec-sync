---
change: split-mcp-implementation-into-cohesive-private-modules
artifact: context
---

# Context

PR810 already split the original MCP monolith into two textual include chunks. Those chunks still share one large Rust namespace. This refactor replaces them with ordinary private modules grouped by protocol, handlers, confinement, preflight, snapshots, manifest traversal and publication. The external MCP wire/API contract is unchanged.

Grok produced the local candidate; a separate GPT reviewer compared original declarations/function bodies and test inventories. That review found a release-test import regression, corrected by direct std::fs imports in the two child test modules. No contributor or primary checkout was changed. PR800 evidence and PR815 check/fix files remain outside this scope.

This package began as a complete unapproved technical draft. Leif directly approved its prepared scope on 2026-10-16, conditional on actual local testing and at least 95% engineering confidence; the native writer recorded actor 0xLeif. The direct response is preserved in the coordinator receipt ci-cleanup-human-approval.json. Implementation review, supported finalization and platform qualification still require their actual evidence.
