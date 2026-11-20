---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: docs
---

# Docs

- **`CHANGELOG.md`, `### Security`:** one entry for these paths, with their versions. It says what a user will notice: a tracked hash cache is not used, a configured directory outside the project is a config error, and absolute links are refused.
- **Specs:**
  - the requirements above, through the deltas;
  - invariants and error cases in `util`, `change`, `cmd_check`, `hash_cache`, `config`, `validator` and `changelog`;
  - the Public API rows for the new `util`, `validator` and `git_utils` functions;
  - `testing.md` and `tasks.md` where tests were added, and a lesson in `util.3md`.
- **`hi/check.md`:** CHECK-1.f, with how it was approved.
- **The private advisory text** (outside this repository) gains the new rows, the Git-directory rule and the confinement root, for Leif to publish.
