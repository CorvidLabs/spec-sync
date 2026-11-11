---
change: read-deadlines-never-panic-on-a-huge-timeout-and-stop-at-every-long-read-phase-including-active-change-loading-and
artifact: testing
---

# Testing

- `a_timeout_too_large_for_the_clock_reads_normally` (integration; panics today on Instant overflow).
- `change_list_past_its_timeout_stops_while_loading_active_changes` (integration; prints a normal verdict today).
- `an_expired_deadline_stops_the_archive_index_before_it_lists_anything` (unit; answers `Ok(0)` today). It uses a worker-style deadline, whose check returns an error instead of exiting.
- Unchanged: `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume`, the archived-read guards, and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-149 | integration tests `a_timeout_too_large_for_the_clock_reads_normally` and `change_list_past_its_timeout_stops_while_loading_active_changes`, unit test `an_expired_deadline_stops_the_archive_index_before_it_lists_anything` (all failing first, `ebe614da`) |
