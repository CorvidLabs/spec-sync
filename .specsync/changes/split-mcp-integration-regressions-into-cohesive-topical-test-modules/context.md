---
change: split-mcp-integration-regressions-into-cohesive-topical-test-modules
artifact: context
---

# Context

The owner requested smaller files across SpecSync, continued the work, and explicitly requested a branch and commit. This isolated branch starts at main 815690d and changes only the 4139-line MCP integration module, its private children, the MCP spec companions, and this lifecycle package. Other active work owns src/change.rs, its unit tests, command lifecycle integration tests, and specs/cmd_change; none of those paths are part of this scope.

Scope approval records the owner's original instructions, transcribed by corvid-agent; it is not an assertion of later human implementation review. No public behavior or new feature is introduced. Retain the real lifecycle at its verified local product checkpoint until a human reviews the implementation; do not claim Accepted, archived, published, or merged status.
