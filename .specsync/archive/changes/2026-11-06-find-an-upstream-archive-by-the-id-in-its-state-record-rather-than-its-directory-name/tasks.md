---
change: find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name
artifact: tasks
---

# Tasks

- [x] Unit test: renamed upstream package is found. Fails on main.
- [x] Read every `*/state.json` under the remote default's archive directory with one `ls-tree -r` and one `cat-file --batch`; match on `id`.
- [x] Keep the #861 spawn-ceiling test green.
- [x] Delta, CHANGELOG entry, requirement evidence (the new helper is private, so no Public API row).
- [x] Lesson in specs/change/change.3md (plane z=18), from the Codex P1 on #943.
