---
change: split-mcp-implementation-into-cohesive-private-modules
artifact: plan
---

# Plan

## Approach

Use a 105-line facade with the existing two external entry points. Move existing bodies into private modules and child test modules; expose sibling helpers only as pub(super). Preserve the old two include_str scan paths as comment-only anchors and scan every actual moved source in the MCP test suite.

## Out of scope

New tools, public wire behavior, security-policy changes, lifecycle implementation, check/fix work, release publication and external provider execution.

## Steps

1. Map declarations, dependencies and platform cfg gates before moving code.
2. Conserve existing test names/behavior and synchronize canonical file ownership and architecture documentation.
3. Independently review the move, correct the release-test imports, and qualify debug tests and release-test compilation.
4. Run all native verification stages and exact committed-range risk/contract/provenance checks before draft publication.
5. Leave real human acceptance, finalization and hosted review/CI gates pending.

## Risks

Cross-module visibility, cfg-only imports, omitted scan coverage and lost tests can change behavior despite a mechanical move. Cross-module cycles already existed as calls in the monolith; moving them does not remove those dependencies. Independent comparison, inventory conservation and runtime regression tests address these risks without claiming formal equivalence.

## Constraints consulted

Existing MCP capability confinement, byte/work budgets, retained-handle identity, atomic publication/rollback and race barriers remain unchanged. The mcp.3md lessons require those guarantees. Cached tools only; latest released SpecSync6.1.1. No overlap with PR800/815.
