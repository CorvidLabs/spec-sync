---
change: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
artifact: requirements
---

# Requirements

This delivers the post-publication step of REQ-github-002 for 6.2.0 and adds no new requirement ID. requirements.md for github stays byte-identical. The only semantic delta is the github Purpose section. After verified stable assets, the main wrapper selects published 6.2.0 by default, and its input, offline published-default declaration, README, Action docs and github module description agree. Immutable v6.2.0 and RC refs keep candidate 40dcf33a and default 6.1.2. Failed or absent consumer gates leave the floating ref unchanged.
