---
change: refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text
artifact: research
---

# Research

- `materialize_change_deltas` (src/change/sdd_version.rs) calls `prepare_module_application`, which skips an item only when its block already equals the delta body. Any other block is replaced by the MODIFIED body, whatever it holds.
- `ensure_approved_delta_bodies_unchanged` (#704/#711) binds the delta BODY to its approval, but nothing records the canonical text the body was written against. So an unedited stale body passes every gate.
- `ChangeRecord.base_commit` is the HEAD when the change was opened, not when it was approved. Using it would misjudge a delta that was written after main had moved. It is also not used, because Leif chose to protect only new approvals.
