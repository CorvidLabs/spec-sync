---
change: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
artifact: tasks
---

# Tasks

- [x] Reproduce: `stale` green and rc=0 while `check` rc=1 on one tree.
- [x] Establish that git CAN measure a committed deletion, so "unmeasurable" is
      the wrong classification and a threshold would bury it.
- [x] Enumerate every call site of the drift primitive before changing any of
      them — five sites, three disguises, found only by enumerating.
- [x] Add one shared predicate and consume it everywhere.
- [x] Make a deletion threshold-independent; keep never-tracked as unmeasurable.
- [x] Close the exit code, not just the message.
- [x] Bring markdown and JSON to parity with the text renderer.
- [x] Stop scoring from claiming a measured zero, without double-charging.
- [x] Pin all of it in sandbox drill 067 with four controls that hold on both
      binaries; confirm it fails 5/5 on the unfixed candidate.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] Whole board unchanged at 48/7.
