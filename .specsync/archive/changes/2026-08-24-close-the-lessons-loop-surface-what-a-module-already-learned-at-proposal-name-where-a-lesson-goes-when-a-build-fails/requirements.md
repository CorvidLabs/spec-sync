---
change: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
artifact: requirements
---

# Requirements

- **REQ-lessons-001** — `change new` names each affected module's `specs/<module>/context.md`
  together with its substantive line count, and says to read it before scoping. A module whose
  context is absent, unreadable, or holds only scaffold prompts produces no line.
- **REQ-lessons-002** — Surfacing is an authoring affordance: it prints on the text path only,
  never in `--json`, and can never fail a lifecycle command.
- **REQ-lessons-003** — A **failed** `change check` names `.specsync/changes/<id>/context.md` as
  where to record what the failure taught. A passing check prints no hint.
- **REQ-lessons-004** — `finalize` writes `lesson-bundle.md` into the archive on a best-effort
  basis. A bundle failure never undoes a completed archival.
- **REQ-lessons-005** — `finalize`'s `next_action` names the fold-back targets and the material
  before naming the merge; with no affected specs it degrades to the plain merge instruction.
- **REQ-lessons-006** — Lessons policy lives in `src/change.rs`. The command layer renders and
  decides nothing, per the thin-dispatch rule in `specs/cmd_change/context.md`.
- **REQ-lessons-007** — Frontmatter stripping has exactly one definition within the change
  module, behaviourally identical to `view::strip_frontmatter`, so repo-wide unification (#696)
  is a no-op rather than a behaviour change.
