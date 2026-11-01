---
change: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
artifact: tasks
---

# Tasks

- [ ] Advance PUBLISHED_ACTION_DEFAULT, the action.yml default and description, and the Action docs default row to 6.1.2.
- [ ] Move archive-gate examples from @main to @v6.1.2 with version '6.1.2'; update README, CHANGELOG and docs/releases/6.1.2.md prose.
- [ ] Apply the github Purpose delta; keep REQ-github-002 and history rows.
- [ ] Run the three release validators, change check --commit, change audit and the pre-push gate.

Owner gates (not tasks): scope approval, implementation review, ship and merge; then omitted-input Linux and macOS Action consumers at the merged commit, then the floating v6 move per docs/RELEASING.md.
