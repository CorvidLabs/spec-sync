---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: tasks
---

# Tasks

- [x] Failing tests on `1b3974e9` with only the tests added: the CLI tests for writes, the repository inputs and the monorepo, two `hash_cache` unit tests, and the write scan
- [x] The work-tree confinement, absolute-link and Git-directory refusals, the safe write open, and the new helpers
- [x] Every writer through the helpers: the change workflow, `--backup`, `init-registry`, `init`, lifecycle, merge, generator, doctor, agents
- [x] The tracked hash cache, `prune`, the directory rule, the confined spec walk, the changelog bound, the import confinement
- [x] The write scan and the wider read scan
- [x] Specs, lesson, `CHANGELOG.md`

Human gates, not tasks: the implementation review before the archive, and the decisions to publish and release.
