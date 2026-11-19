---
change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
artifact: design
---

# Design

- `is_definite_absence(args, code)`: exit 1 from `is_config_read` args, from `symbolic-ref -q ...`,
  or from `rev-parse --verify --quiet ...`. `note_repository_git` memoizes only a success or a
  definite absence.
- `is_repository_fact` also matches `symbolic-ref -q --short <ref>`.
- `remote_default_ref` (`src/change/validate_loaded_change.rs`) asks
  `symbolic-ref -q --short refs/remotes/origin/HEAD`, then `rev-parse --verify --quiet origin/main`
  and then `origin/master`. The answers are the same; only the exit code of an absent ref changes,
  from 128 to 1.
