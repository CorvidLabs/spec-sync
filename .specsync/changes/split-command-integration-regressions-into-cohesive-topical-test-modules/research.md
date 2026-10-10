---
change: split-command-integration-regressions-into-cohesive-topical-test-modules
artifact: research
---

# Research

The 5496-line test module is larger than the next unblocked production candidates and already contains clear command/domain sections. The first issue block and retained coverage block each need topical subdivision; generation/companion tests can share one owner, setup/hooks another, and spec maintenance can group compact/archive/merge without tiny per-command files. The harness registers commands.rs by path and its flat-file orphan guard must remain intact. Baseline log proves all 120 Linux commands:: tests pass; additional platform-only declarations will be counted mechanically. No runtime or production modification is needed.
