---
change: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
artifact: requirements
---

# Requirements

This delivers the post-publication step of REQ-github-002 for 6.1.2 and adds no new requirement ID. requirements.md for github stays byte-identical. The only semantic delta is the github Purpose section. After verified stable assets, the main wrapper selects published 6.1.2 by default, and its input, offline published-default declaration, README, Action docs and github module description agree. Immutable v6.1.2 and RC refs keep candidate d08d2746 and default 6.1.1. Failed or absent consumer gates leave the floating ref unchanged.
