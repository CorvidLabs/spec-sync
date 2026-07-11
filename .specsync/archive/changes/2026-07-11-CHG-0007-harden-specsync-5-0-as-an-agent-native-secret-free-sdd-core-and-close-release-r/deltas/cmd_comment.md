## ADDED

### REQUIREMENT REQ-cmd-comment-003
The comment command SHALL emit a bounded markdown protocol on stdout.

Acceptance Criteria
- Configured SDD verification command output is captured away from stdout in comment mode.
- Only the final markdown report is printed when `--pr` is omitted.
- Oversized detail is truncated with a clear remediation message before GitHub's comment limit.
