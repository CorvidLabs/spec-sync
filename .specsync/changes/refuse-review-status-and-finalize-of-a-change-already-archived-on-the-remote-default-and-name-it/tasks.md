---
change: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
artifact: tasks
---

# Tasks

- [ ] Integration test: upstream archive present -> review refuses before acceptance with the named cause; status and ship-status not safe; finalize refuses with the same cause. Fails first on main.
- [ ] Integration test: no upstream archive -> squash, review, finalize archives (#690 guard).
- [ ] Shared check over `archive_introduction_index` returning reference and commit.
- [ ] Wire it into review, handoff/status, ship-status and finalize.
- [ ] Deltas, CHANGELOG entry, requirement evidence.
