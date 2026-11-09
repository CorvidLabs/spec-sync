---
change: read-deadlines-never-panic-on-a-huge-timeout-and-stop-at-every-long-read-phase-including-active-change-loading-and
artifact: context
---

# Context

G-deadline in Leif's burn-down (`_audits/specsync-triage-2026-11-06.md`): #924, #925 and #954, three Codex findings on #918's read deadline (REQ-change-137) that were deferred. REQ-change-149, assigned by the orc. HI: SPEED-1.

- **#925.** `begin_read_deadline` computes `Instant::now() + Duration::from_secs(limit)`, which panics for a value such as `u64::MAX`.
- **#924.** Loading active changes (`list_changes_uncached`) and the per-commit workflow-version history walk have no deadline boundary, so `change list` and friends overrun `--timeout` and print a normal verdict.
- **#954.** The archive-introduction index starts its bulk `diff-tree --stdin` before its first check, and with no commit to loop over it never checks at all.
