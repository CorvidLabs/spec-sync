---
change: split-mcp-implementation-into-cohesive-private-modules
artifact: research
---

# Research

The baseline has123 macOS MCP unit tests and72 MCP integration cases. The candidate retains those tests, adds one complete moved-source scan (124 unit tests), and leaves integration/mcp.rs unchanged. The independent source review conserved all125 source-declared original tests, including platform-specific cases unavailable on this host; this is not a formal AST equivalence proof.

Two launcher-environment issues were reproduced without changing production guards or tests. CI=true without GITHUB_WORKSPACE makes local synthetic fixtures take protected CI paths. Separately, forced color overrides add ANSI bytes to integration assertions expecting plain strings. Same-code controlled checks fail in the inherited modes and pass in the ordinary local fixture environment. All2583 unit tests and462 integration tests then pass. Actual hosted CI and lifecycle approval are separate gates.

The source refactor itself introduced an fs import defect for release-profile test compilation. The identical check failed101 before direct imports and passed0 afterward. This check compiles the release test target; it does not execute release-profile tests.
