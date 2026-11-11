---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: tasks
---

# Tasks

- [ ] After Leif approves the definition, write the red-first integration tests in testing.md
- [ ] Point approved and implementing `next_action` at `change check <id> --commit` in the domain and in text status/show
- [ ] Update REQ-change-132 callers and unit tests that pin the bare check command
- [ ] Refuse `change review` and `change ship` when the verification commit lacks the change's work
- [ ] Insert change.spec invariant 59 in numeric place
- [ ] Re-run the #535 reproduction and the new tests through the branch build
