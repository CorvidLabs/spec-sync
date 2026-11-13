---
change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
artifact: context
---

# Context

#434 (filed on 5.2.0). Part 1, unknown fields dropped on rewrite, was fixed by #1044. Re-tested on
main @ 18385ef5e on 2026-11-13:
- `state.json` with `schema_version: 99` is read silently.
- `verification-attempts.json` with `schema_version: 99` draws nothing from `change audit`,
  `change status` or `doctor`, while `change check` refuses it.
- A type error in `approvals.json` prints `invalid type: ...` with no file named.

Leif chose to build both rules on 2026-11-13 ("Yes (Recommended)", coordinator chat 04:39Z, relayed
by claude-code-t370). The coordinator allocated REQ-change-164 and REQ-change-165. The wording is
with the coordinator for an alignment check.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
