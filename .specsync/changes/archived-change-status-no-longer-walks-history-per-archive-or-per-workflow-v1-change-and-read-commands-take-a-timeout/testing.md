---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: testing
---

# Testing

Gating, counted in Git processes, so independent of the machine:
- `warm_archived_status_git_processes_do_not_grow_with_legacy_changes` (new; fails now: 97 with 2 workflow-v1 changes, 115 with 8).
- A cold archived-status guard over archive-introducing commits (new; failing first).
- Equivalence: the one-walk index equals the per-commit index; workflow-v1 cutoff verdicts (absent, added after the cutoff, archived after it, a different id) equal today's.
- Timeout: `--timeout 0` on an archived status exits 124 with the JSON shape, every change file is byte-identical, and the verdicts completed so far are in the cache. The resume command gives the same verdict.
- The existing `fast_archived_reads` and `fast_finalize` guards, the hidden-downgrade tests and the full verify lane all pass unchanged.

On demand: `fledge run bench-archived-reads` with workflow-v1 changes in the fixture, against under 2 s warm and under 5 s cold on an unloaded Mac. This repository's archived status is timed before and after.
