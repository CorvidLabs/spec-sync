---
change: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
artifact: tasks
---

# Tasks

- [x] Integration test: upstream archive present -> review refuses before acceptance with the named cause; status and ship-status not safe; finalize refuses with the same cause. Fails first on main.
- [x] Integration test: no upstream archive -> squash, review, finalize archives (#690 guard).
- [x] Shared check over `archive_introduction_index` returning reference and commit.
- [x] Wire it into review, handoff/status, ship-status and finalize.
- [x] Deltas, CHANGELOG entry, requirement evidence.
