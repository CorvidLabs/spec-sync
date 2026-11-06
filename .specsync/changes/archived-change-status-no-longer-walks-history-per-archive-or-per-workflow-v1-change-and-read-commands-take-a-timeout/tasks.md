---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: tasks
---

# Tasks

- [x] Failing warm guard committed (workflow-v1 changes)
- [x] Failing cold guard (archive-introducing commits), split into the index guard (A) and the worker guard (C)
- [x] B: cutoff-tree verdict + persistent cache + equivalence tests
- [x] A: one-walk introduction index + equivalence test
- [ ] C: seeded read-only scopes for prefetch workers + worker guard + git_utils unit tests
- [ ] Deadline module, CLI flag, env var, exit 124, text/JSON, cache flush; tests first
- [ ] Benchmark fixture with workflow-v1 changes; record before/after times
- [ ] Docs, CHANGELOG, spec Public API rows
