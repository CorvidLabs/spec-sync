---
change: never-write-a-project-file-through-a-symbolic-link
artifact: requirements
---

# Requirements

Human intent SETUP-6: "A command I run in a repository somebody else wrote never writes outside it through a link it found there."

Added, one per module, in `deltas/`: `REQ-util-002` (the shared rule), `REQ-hooks-006`, `REQ-agents-011`, `REQ-mcp-011`, `REQ-registry-003`, `REQ-cmd-resolve-002` (strict: no link at all), `REQ-cmd-migrate-005`, `REQ-generator-010`, `REQ-cmd-scaffold-007`, `REQ-cmd-new-006`, `REQ-cmd-wizard-003`, `REQ-cmd-import-003`.
