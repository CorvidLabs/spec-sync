## ADDED

### REQUIREMENT REQ-change-149

A read deadline (REQ-change-137) SHALL never panic: a timeout too large to add to the clock SHALL mean no deadline. Loading active changes, each commit of the workflow-version history walk, and the start of the archive-introduction index's bulk listing SHALL each be a phase boundary where a read past its deadline stops, so a read SHALL NOT overrun its deadline through any of them and print a normal verdict (SPEED-1).

Acceptance Criteria
- `--timeout` or `SPECSYNC_TIMEOUT` of `18446744073709551615` reads normally.
- `change list --timeout 0` with active changes exits 124 during `active change loading`.
- An expired deadline stops the archive-introduction index before it lists anything, even with no archive-introducing commit.
