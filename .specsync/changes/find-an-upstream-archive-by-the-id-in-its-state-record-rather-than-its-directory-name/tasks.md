---
change: find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name
artifact: tasks
---

# Tasks

- [x] Unit test: renamed upstream package is found. Fails on main.
- [ ] Read every `*/state.json` under the remote default's archive directory with one `ls-tree -r` and one `cat-file --batch`; match on `id`.
- [ ] Keep the #861 spawn-ceiling test green.
- [ ] Delta, Public API row, CHANGELOG entry, requirement evidence.
