---
change: split-command-integration-regressions-into-cohesive-topical-test-modules
artifact: design
---

# Design

The existing commands module remains a small facade declaring private topical children. Candidate owners are issues with confinement child, coverage with race/input children, generation, project setup/hooks, history/diff, migration, dependency analysis and spec maintenance. Exact grouping should follow helper ownership and keep each file below 1000 lines without arbitrary fragments. Shared specsync_process and platform junction fixtures stay beneath the commands module with minimal parent-visible seams. CLI documentation lists the new navigation paths; its production file mapping and behavioral contract remain unchanged. Compare compiled test names ignoring their newly qualified module prefix, preserving count and cfgs.
