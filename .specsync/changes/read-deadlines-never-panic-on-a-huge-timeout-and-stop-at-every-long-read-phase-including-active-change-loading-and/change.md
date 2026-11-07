---
id: read-deadlines-never-panic-on-a-huge-timeout-and-stop-at-every-long-read-phase-including-active-change-loading-and
state: implementing
type: bug_fix
base_commit: ebe614dad6005282e805ed2eba378a3015d9418f
---

# Read deadlines never panic on a huge timeout and stop at every long read phase, including active-change loading and before the bulk archive listing

## Intent

Read deadlines never panic on a huge timeout and stop at every long read phase, including active-change loading and before the bulk archive listing

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a_timeout_too_large_for_the_clock_reads_normally passes: --timeout or SPECSYNC_TIMEOUT too large to add to the clock means no deadline, never a panic (failing first in ebe614da)
- change_list_past_its_timeout_stops_while_loading_active_changes passes: loading active changes is a phase with a deadline boundary (active change loading), so a read past its deadline exits 124 with that phase (failing first in ebe614da)
- an_expired_deadline_stops_the_archive_index_before_it_lists_anything passes: the archive-introduction index checks its deadline before its bulk diff-tree listing, including when there is no commit to loop over (failing first in ebe614da)
- The workflow-version history walk checks the deadline between commits; a worker that sees it stops and the command thread reports, as REQ-change-137 already does
- Without a deadline nothing changes; the report shape, exit 124, the cache flush and every refusal are unchanged

## No-spec Rationale

Not applicable
