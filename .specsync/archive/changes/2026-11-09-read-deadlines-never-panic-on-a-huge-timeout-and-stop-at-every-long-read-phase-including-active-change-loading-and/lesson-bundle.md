# Lesson bundle: read-deadlines-never-panic-on-a-huge-timeout-and-stop-at-every-long-read-phase-including-active-change-loading-and

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Read deadlines never panic on a huge timeout and stop at every long read phase, including active-change loading and before the bulk archive listing
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/read_deadline.rs, src/change/sdd_version.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change_tests/fast_archived_reads.rs, tests/integration/change.rs, specs/change, CHANGELOG.md
- **Acceptance**: a_timeout_too_large_for_the_clock_reads_normally passes: --timeout or SPECSYNC_TIMEOUT too large to add to the clock means no deadline, never a panic (failing first in ebe614da)
- **Acceptance**: change_list_past_its_timeout_stops_while_loading_active_changes passes: loading active changes is a phase with a deadline boundary (active change loading), so a read past its deadline exits 124 with that phase (failing first in ebe614da)
- **Acceptance**: an_expired_deadline_stops_the_archive_index_before_it_lists_anything passes: the archive-introduction index checks its deadline before its bulk diff-tree listing, including when there is no commit to loop over (failing first in ebe614da)
- **Acceptance**: The workflow-version history walk checks the deadline between commits; a worker that sees it stops and the command thread reports, as REQ-change-137 already does
- **Acceptance**: Without a deadline nothing changes; the report shape, exit 124, the cache flush and every refusal are unchanged

## Evidence

- Verification commit: `76d40398e11d6bd333caec031970ddd92bd21fba`
- Base commit: `ebe614dad6005282e805ed2eba378a3015d9418f`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

G-deadline in Leif's burn-down (`_audits/specsync-triage-2026-11-06.md`): #924, #925 and #954, three Codex findings on #918's read deadline (REQ-change-137) that were deferred. REQ-change-149, assigned by the orc. HI: SPEED-1.

- **#925.** `begin_read_deadline` computes `Instant::now() + Duration::from_secs(limit)`, which panics for a value such as `u64::MAX`.
- **#924.** Loading active changes (`list_changes_uncached`) and the per-commit workflow-version history walk have no deadline boundary, so `change list` and friends overrun `--timeout` and print a normal verdict.
- **#954.** The archive-introduction index starts its bulk `diff-tree --stdin` before its first check, and with no commit to loop over it never checks at all.

## From the change's design.md

# Design

- **#925.** `begin_read_deadline` uses `Instant::now().checked_add(..)`. A limit the clock can't hold sets no deadline, the same as no `--timeout`, because that limit is longer than any read.
- **#924.** `list_changes_uncached` calls `check_read_deadline("active change loading", done, total)` before each workspace. `validate_workflow_version_history_uncached` calls `check_read_deadline("workflow-version history", done, total)` before each commit; on a worker that is an error the worker stops on, and the command thread reports.
- **#954.** `archive_introduction_index` calls `check_read_deadline("archive-introduction index", 0, total)` before `archive_introduction_listings`.

The report, exit 124 and the cache flush are REQ-change-137's, unchanged.

## From the change's testing.md

# Testing

- `a_timeout_too_large_for_the_clock_reads_normally` (integration; panics today on Instant overflow).
- `change_list_past_its_timeout_stops_while_loading_active_changes` (integration; prints a normal verdict today).
- `an_expired_deadline_stops_the_archive_index_before_it_lists_anything` (unit; answers `Ok(0)` today). It uses a worker-style deadline, whose check returns an error instead of exiting.
- Unchanged: `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume`, the archived-read guards, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-149 | integration tests `a_timeout_too_large_for_the_clock_reads_normally` and `change_list_past_its_timeout_stops_while_loading_active_changes`, unit test `an_expired_deadline_stops_the_archive_index_before_it_lists_anything` (all failing first, `ebe614da`) |

## Where these lessons go

- `specs/change/change.3md`
