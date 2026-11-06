---
change: declare-the-three-paths-the-archived-status-latency-change-edited-but-did-not-list
artifact: context
---

# Context

#918 shipped `archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout` at archive tip `76ee339b`. CI's Lifecycle gate and trust run `specsync change audit --strict`, which failed: "meaningful changed paths are not covered by an active change: src/change.rs, src/change/sdd_version.rs, tests/integration/change.rs". The archived change declared ten paths but also edited these three:

- `src/change.rs`, +1: the `include!` for `change/read_deadline.rs`.
- `src/change/sdd_version.rs`, +11: `flush_change_read_scope_caches`, which flushes read-scope caches when a read times out.
- `tests/integration/change.rs`, +116: `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume`.

All three are in the PR diff Leif reviewed, and they implement REQ-change-137 and REQ-cmd-change-032. `change correct` can't amend affected paths, so Leif chose this covering change ("Same covering fix", relayed by orc t18 on 2026-11-06). It changes no code, tests or spec text.
