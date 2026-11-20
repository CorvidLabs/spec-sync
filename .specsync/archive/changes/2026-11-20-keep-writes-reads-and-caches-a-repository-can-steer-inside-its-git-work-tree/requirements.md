---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: requirements
---

# Requirements

Human intent:
- SETUP-6: "A command I run in a repository somebody else wrote never writes outside it through a link it found there."
- SETUP-7: "A command I run in a repository somebody else wrote never reads outside it through a link it found there, and no file there can make it hang or run out of memory."
- CHECK-1.f (new, approved as written): "A result that a repository somebody else wrote carries along never hides a failure my check would report."

Added, one delta per module in `deltas/`:
- **Confinement and refusals:** `REQ-util-004`, `REQ-util-005`.
- **The change workflow:** `REQ-change-163`.
- **`check`:** `REQ-cmd-check-020`, `REQ-cmd-check-021`.
- **The hash cache:** `REQ-hash-cache-009`, `REQ-hash-cache-010`.
- **Config:** `REQ-config-017`.
- **Spec discovery and reads:** `REQ-validator-050`, `REQ-validator-051`.
- **The changelog bound:** `REQ-changelog-002`, `REQ-git-utils-012`, `REQ-git-utils-013`.
- **Imports, the registry, agents:** `REQ-cmd-import-004`, `REQ-cmd-init-registry-004`, `REQ-agents-013`.
- **Modules the write scan holds to the helpers:** `REQ-archive-003`, `REQ-compact-003`, `REQ-merge-008`, `REQ-cmd-merge-006`, `REQ-cmd-lifecycle-006`, `REQ-cmd-init-010`, `REQ-generator-012`, `REQ-cmd-generate-005`, `REQ-cmd-issues-004`, `REQ-hooks-008`, `REQ-mcp-013`, `REQ-commands-015`, `REQ-rehash-002`, `REQ-cmd-view-003`, `REQ-deps-005`, `REQ-cmd-migrate-007`.
