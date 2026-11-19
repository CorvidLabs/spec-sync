---
change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
artifact: context
---

# Context

#887, from the Codex review of #869. `note_repository_git` (`src/git_utils.rs`) memoized any git
fact that exited with a code, nonzero included. REQ-git-utils-009 says "A failed git query SHALL
NOT be memoized", and its acceptance criteria say a failure is retried on the next read. #869
measured that caching only exit 0 breaks REQ-change-127's spawn ceiling: the slowest lifecycle
test rose from 5,996 to 7,560 spawns, against a ceiling of 6,225. Most of the repeats were
definite "absent" answers, not failures.

This takes the issue's option 1, which keeps the requirement's wording. Exit 0 is memoized, and
so is exit 1 from a `git config` lookup (the key is unset), from `symbolic-ref -q` and from
`rev-parse --verify --quiet` (the ref is absent). `remote_default_ref` switches to those quiet
forms, and `symbolic-ref -q --short` becomes a recognised repository fact. Every other nonzero exit
is a failure and is asked again. Exit 128 from `rev-parse --verify HEAD` before any commit is one
example.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
