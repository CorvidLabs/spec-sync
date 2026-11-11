---
change: identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists
artifact: tasks
---

# Tasks

- [x] Trim a relayed approval's decider like its actor
- [x] Retire, doctor late close and reconcile-history (at write) use `normalize_actor`; reopen checks before moving
- [x] Refuse Unicode line and paragraph separators and bidi overrides or isolates in actors
- [x] Refuse empty-list answers; `none` clears `affected_specs` and `dependencies`
- [x] Name the canonical path, the off-path spec, or the unreadable registry in `affected_specs` refusals
- [x] REQ-change-150 delta, invariant 52 and the z=46 lesson match the code
- [x] Tests: relay with a padded actor, empty lists and `none`, Unicode actors
