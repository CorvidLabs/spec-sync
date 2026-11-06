---
change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
artifact: tasks
---

# Tasks

- [x] Integration test (workflow v2): corrupt archive is `not-yet` in JSON and text, names the restore, the named command recovers; committed damage recovers from the archiving commit. Fails first on main ad50bdef.
- [x] `CorruptArchive` signal, `classify_handoff` arm, `corrupt_archive_recovery`, `corrupt_archive_recovery_for_guidance`.
- [x] Wire into `summarize_change` (next action + handoff), `handoff_summary` (archived reads terminal evidence) and `text_mode_next_action`.
- [x] Unit tests for every new classifier branch; update pinned next-action strings.
- [x] Specs: Public API rows, invariant 42, version bumps; HI CHANGE-7.a.1 and CHANGE-7.a.2 once confirmed; CHANGELOG.
- [x] Measure text status cost before and after; file the roster-walk performance follow-up (#897).
