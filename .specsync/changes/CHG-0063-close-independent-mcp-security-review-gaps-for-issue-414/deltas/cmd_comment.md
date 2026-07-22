## MODIFIED

### REQUIREMENT REQ-cmd-comment-003

The comment command SHALL emit a bounded markdown protocol on stdout.

Acceptance Criteria

- Configured SDD verification command output is captured away from stdout in comment mode.
- Only the final markdown report is printed when `--pr` is omitted.
- Oversized detail is truncated with a clear remediation message before GitHub's comment limit.
- Malformed Gradle/manifest discovery exits nonzero with an explicit inconclusive stderr diagnostic
  before any misleading markdown is rendered or posted.

### SPEC SECTION Invariants

7. Coverage uses checked manifest discovery; malformed Gradle settings produce an inconclusive
   stderr diagnostic and exit 1 before a misleading PR summary can be emitted.
