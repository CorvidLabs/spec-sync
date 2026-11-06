---
change: stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent
artifact: tasks
---

# Tasks

- [x] Unit and integration tests with the archived record's blob removed. Both fail on a6116e0c.
- [ ] `UpstreamArchive` gains a cannot-tell case; every Git failure on the lookup after the remote default resolves returns it instead of `None`.
- [ ] The message for it names the reference and the recovery (fetch, then list the archived packages for this id).
- [ ] Delta, Public API rows, CHANGELOG entry, module task and lesson, requirement evidence.
