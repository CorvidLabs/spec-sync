---
change: ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so
artifact: requirements
---

# Requirements

- **REQ-lessons-008** — `change ship` names the lesson fold-back targets and the archived bundle
  path before its remaining guidance, in both text and JSON, whenever the change owns at least
  one affected spec.
- **REQ-lessons-009** — The fold-back is named FIRST. It is the step a merge makes irreversible:
  after the merge the change is inert history and its material is archived where nobody reads it.
- **REQ-lessons-010** — A change owning no affected specs receives guidance byte-identical to
  the guidance before this change, in every push/wait combination.
- **REQ-lessons-011** — Sibling-change blockers survive the fold-back prefix; a fold-back must
  never displace "do not merge while any change is active".
- **REQ-lessons-012** — Post-finalize guidance is composed by one pure function per exit, not by
  a string duplicated per verb.
