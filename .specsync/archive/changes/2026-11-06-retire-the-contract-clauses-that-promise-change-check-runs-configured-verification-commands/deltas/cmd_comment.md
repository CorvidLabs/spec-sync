# Retire the comment contract clauses that promise configured verification commands run

## MODIFIED

### REQUIREMENT REQ-cmd-comment-003

The comment command SHALL emit a bounded markdown protocol on stdout.

Acceptance Criteria

- `comment` executes no configured SDD verification command (REQ-cmd-comment-004, REQ-change-023), so no child output can reach stdout.
- Only the final markdown report is printed when `--pr` is omitted.
- Oversized detail is truncated with a clear remediation message before GitHub's comment limit.
- Malformed Gradle/manifest discovery exits nonzero with an explicit inconclusive stderr diagnostic
  before any misleading markdown is rendered or posted.

### SPEC SECTION Behavioral Examples

### Scenario: Print to stdout

- **Given** `--pr` is not set
- **When** `cmd_comment` runs
- **Then** prints markdown summary to stdout

### Scenario: Post to PR

- **Given** `--pr 42` is set
- **When** `cmd_comment` runs
- **Then** posts comment on PR #42

### Scenario: Marketplace action captures stdout

- **Given** the marketplace action runs with `comment: true`
- **When** `specsync comment` is invoked without `--pr`
- **Then** the stdout output is identical to what the CI workflow captures via `cargo run -- comment`

### Scenario: Configured verification command is not executed

- **Given** `.specsync/sdd.json` lists an SDD verification command that would write to stdout or stderr
- **When** `specsync comment` runs
- **Then** the command is not executed, the verdict is the spec-check verdict alone, and nothing from the command appears in the rendered markdown stream

## REMOVED

### REQUIREMENT REQ-cmd-comment-001

Retired (#848). It required comments to fold SDD lifecycle failures into their status and
remediation details. REQ-cmd-comment-004 reverses it: `comment` reports spec-check results only and
lifecycle state does not contribute to its totals or exit status. That severance (#543) is also why
`comment` executes no configured verification command.

### REQUIREMENT REQ-cmd-comment-002

Retired (#848). It required SDD lifecycle failures to be reported even with no canonical spec, which
is the same lifecycle folding REQ-cmd-comment-004 reverses. No test asserts it; the SDD-only-failure
tests were removed with the severance (#543).
