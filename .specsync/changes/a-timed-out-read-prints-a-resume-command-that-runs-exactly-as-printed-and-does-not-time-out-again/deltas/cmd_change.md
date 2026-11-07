## ADDED

### REQUIREMENT REQ-cmd-change-036

The resume command a timed-out read prints (REQ-cmd-change-032) SHALL split back, under a POSIX shell, into exactly the arguments of the original invocation without `--timeout`: every argument not made only of shell-safe characters SHALL be single-quoted. When `SPECSYNC_TIMEOUT` was set, the command SHALL begin with `SPECSYNC_TIMEOUT= ` so the resumed run has no deadline (SPEED-1).

Acceptance Criteria
- A resume command for arguments holding `$`, `;`, `*`, `'`, spaces and an empty argument splits back into exactly those arguments.
- With `SPECSYNC_TIMEOUT` set, the resume command starts with `SPECSYNC_TIMEOUT= specsync `; without it, it starts with `specsync `.
