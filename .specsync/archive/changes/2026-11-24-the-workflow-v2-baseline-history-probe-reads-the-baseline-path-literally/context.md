---
change: the-workflow-v2-baseline-history-probe-reads-the-baseline-path-literally
artifact: context
---

# Context

#1074, found while writing #990's test. `workflow_v2_baseline_exists_in_reachable_history`
(`src/change/validate_loaded_change.rs`) ran `rev-list --full-history --max-count=1 HEAD -- <path>`
with the repository-relative baseline path as a plain pathspec. For a project at `:(exclude)app/`
Git read it as an exclude that matched every other commit, so a never-committed baseline was
reported as "committed workflow-v2 baseline was deleted". The fix passes
`:(top,literal)<baseline path>`, as #851 and #990 did for the module's other history queries.
Leif approved it as a bug fix on 2026-11-13 ("#1074 + #1075 (new)").

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
