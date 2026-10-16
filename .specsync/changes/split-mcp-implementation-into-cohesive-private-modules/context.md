---
change: split-mcp-implementation-into-cohesive-private-modules
artifact: context
---

# Context

PR810 already split the original MCP monolith into two textual include chunks. Those chunks still share one large Rust namespace. This refactor replaces them with ordinary private modules grouped by protocol, handlers, confinement, preflight, snapshots, manifest traversal and publication. The external MCP wire/API contract is unchanged.

Grok produced the local candidate; a separate GPT reviewer compared original declarations/function bodies and test inventories. That review found a release-test import regression, corrected by direct std::fs imports in the two child test modules. No contributor or primary checkout was changed. PR800 evidence and PR815 check/fix files remain outside this scope.

This package is a complete technical draft. It records no human definition approval, implementation acceptance, lifecycle review or platform qualification.
