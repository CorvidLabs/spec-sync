---
change: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
artifact: tasks
---

# Tasks

- [x] Advance PUBLISHED_ACTION_DEFAULT, the action.yml default and description, and the Action docs default row to 6.2.0.
- [x] Refresh README, SECURITY, MIGRATION, ADOPTING, site guidance, CHANGELOG and docs/releases/6.2.0.md.
- [x] Apply the github Purpose delta and companions; keep REQ-github-002 and history rows.
- [x] Run the three release validators (all pass; the 66 release-validator tests needed no fixture change).

Owner gates (not tasks): scope approval, implementation review, ship and merge; then omitted-input Linux and macOS Action consumers at the merged commit, then the floating v6 move per docs/RELEASING.md.
