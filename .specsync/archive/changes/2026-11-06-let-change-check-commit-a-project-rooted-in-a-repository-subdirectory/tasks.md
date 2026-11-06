---
change: let-change-check-commit-a-project-rooted-in-a-repository-subdirectory
artifact: tasks
---

# Tasks

- [x] Integration test: project in `app/`, one change, `change check --commit` completes and commits the materialized spec. Fails first on main.
- [x] Unit test: a repository-relative path is admitted after the prefix is removed; a path outside the prefix and a non-candidate path are still refused.
- [x] Strip the repository prefix from `diff-files` output before the scope check.
- [x] Delta, CHANGELOG entry, requirement evidence.
