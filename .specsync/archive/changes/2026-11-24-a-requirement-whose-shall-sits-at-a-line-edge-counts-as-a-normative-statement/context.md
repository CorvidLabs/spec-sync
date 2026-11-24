---
change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
artifact: context
---

# Context

#1076, found while recording #434's REQ-change-164: `validate_requirement`
(`src/change/collect_requirement_ids.rs`) accepted a requirement only when it contained `" SHALL "` or started
with `"SHALL "`. A requirement wrapped at a normal width, with SHALL at a line edge, was refused at
`change approve` as having no normative SHALL. Now SHALL counts as a whole word across any whitespace.
Lane A (t38c) agreed this lane takes it.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through a
Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
