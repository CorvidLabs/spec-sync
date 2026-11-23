---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: requirements
---

# Requirements

Human intent:
- SETUP-6: "A command I run in a repository somebody else wrote never writes outside it through a link it found there."
- SETUP-7: "A command I run in a repository somebody else wrote never reads outside it through a link it found there, and no file there can make it hang or run out of memory."
- CHECK-1.f: "A result that a repository somebody else wrote carries along never hides a failure my check would report."

One delta per module in `deltas/`:
- **Local state, confinement, scans:** added `REQ-util-006`, `REQ-util-007` and `REQ-util-008`.
- **The hash cache:** `REQ-hash-cache-009` modified.
- **`check`:** `REQ-cmd-check-021` modified.
- **`rehash`:** added `REQ-rehash-003`.
- **The journal:** `REQ-change-162` modified.
- **The remote spec cache:** `REQ-cmd-resolve-002` and `REQ-cmd-resolve-003` modified.
- **The hooks folder:** added `REQ-hooks-009`.
- **Coverage's spec reads:** added `REQ-validator-052`.
