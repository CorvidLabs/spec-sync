---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: tasks
---

# Tasks

- [x] Failing test: `bounded_git_read_returns_while_a_descendant_holds_the_output_pipe` waited 15 seconds with the holder still alive before the fix.
- [x] Return from the bounded Git reader and from `output_after_child_exits` when the direct child exits.
- [x] Point status Git children at the null device.
- [x] Record REQ-git-utils-008, REQ-change-126, and SPEED-1.e.
- [ ] Leif records `change approve` and `change review` for this change. This session does not.
