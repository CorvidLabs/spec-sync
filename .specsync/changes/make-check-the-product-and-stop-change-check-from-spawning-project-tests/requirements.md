---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: requirements
---

# Requirements

This change rewrites existing requirements rather than minting new IDs.

- `REQ-change-023`, `REQ-change-049`, `REQ-change-050`, `REQ-change-058`, `REQ-change-091`
  — `change check` is in-process spec↔code sync and does not spawn project commands.
- `REQ-cmd-check-004` — `specsync check` does not walk SDD.
- `REQ-cmd-init-005` stays: init still bootstraps so the next check is clean. Fresh init
  now writes SDD off (`REQ-change-050`).
- `REQ-agents-check-audit-commands-001` — generated check skill describes spec↔code sync.
