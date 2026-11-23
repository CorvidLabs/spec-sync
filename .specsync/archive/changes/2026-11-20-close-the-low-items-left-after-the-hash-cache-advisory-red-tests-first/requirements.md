---
change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
artifact: requirements
---

# Requirements

Human intent: SETUP-6, SETUP-7, SETUP-8 and CHECK-1.f.

Added:
- `REQ-util-011`: `read_typed_file`, for paths the person types.
- `REQ-cmd-change-041`: `correct-owner --manifest` through it.
- `REQ-cmd-scaffold-009`: `scaffold --template` through it.

Modified:
- `REQ-util-006`: the journal in the state folder (S5), N2, L2, one clause removed.
- `REQ-util-008`: the widened scans (X1), one clause removed.
- `REQ-util-009`: the read cache through `LocalState` (C1).
- `REQ-util-010`: record-folder listings (W1).
- `REQ-change-162`: the journal records its project (S2, S5), one clause removed.
- `REQ-hooks-009`: S1, X2 and H1, one clause removed.
- `REQ-hash-cache-007`: a cache over the bound is refused (C5).
- `REQ-cmd-check-021`: a refused state folder, one clause removed.
- `REQ-commands-014`, `REQ-cmd-resolve-003`, `REQ-cmd-stale-005`: a refused spec read is reported, never dropped (C2 to C4).
- `REQ-validator-053`: record-folder listings (W1).
