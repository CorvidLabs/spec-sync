---
change: let-change-check-commit-a-project-rooted-in-a-repository-subdirectory
artifact: tasks
---

# Tasks

- [ ] Integration test: project in `app/`, one change, `change check --commit` completes and commits the materialized spec. Fails first on main.
- [ ] Unit test: a repository-relative path is admitted after the prefix is removed; a path outside the prefix and a non-candidate path are still refused.
- [ ] Strip the repository prefix from `diff-files` output before the scope check.
- [ ] Delta, CHANGELOG entry, requirement evidence.
