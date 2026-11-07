---
change: read-deadlines-never-panic-on-a-huge-timeout-and-stop-at-every-long-read-phase-including-active-change-loading-and
artifact: design
---

# Design

- **#925.** `begin_read_deadline` uses `Instant::now().checked_add(..)`. A limit the clock can't hold sets no deadline, the same as no `--timeout`, because that limit is longer than any read.
- **#924.** `list_changes_uncached` calls `check_read_deadline("active change loading", done, total)` before each workspace. `validate_workflow_version_history_uncached` calls `check_read_deadline("workflow-version history", done, total)` before each commit; on a worker that is an error the worker stops on, and the command thread reports.
- **#954.** `archive_introduction_index` calls `check_read_deadline("archive-introduction index", 0, total)` before `archive_introduction_listings`.

The report, exit 124 and the cache flush are REQ-change-137's, unchanged.
