---
change: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven
artifact: research
---

# Research

Raven at CorvidLabs/raven main 74e43005, Linux, 8 cores. A logging Git shim counted processes: 9,972 for one archived status read on main f82eb09, 2,838 after lazy terminal evidence, 1,685 after scope memos, and 506 once the persistent cache is warm. Of the 1,685, most were per-archive workflow-version history walks (a glob `rev-list`, a `log --name-only` and a `rev-list` over the state paths, then one `show` per touching commit and parent) and 81 `diff-tree` plus 250 `show` calls rebuilding archive introductions. Related issues: #831 (cheap archived read for CI), #645 (approve process count) and #439 (lifecycle performance cliffs).
