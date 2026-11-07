---
change: a-timed-out-read-prints-a-resume-command-that-runs-exactly-as-printed-and-does-not-time-out-again
artifact: testing
---

# Testing

- `the_resume_command_splits_back_into_the_original_arguments` (unit, new; fails today because `it's` is printed bare, so `sh` can't even parse the command).
- `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (integration, extended; fails today: with `SPECSYNC_TIMEOUT=0` the resume command doesn't clear it).
- Unchanged: the rest of that integration test (`--timeout 0`, `--timeout=0`, the text report, byte-identical `.specsync/`), and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-036 | unit test `the_resume_command_splits_back_into_the_original_arguments` and integration test `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (both failing first, `db16db8a`) |
