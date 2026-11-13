---
id: doctor-fix-refuses-under-an-inherited-verification-context-and-read-only-doctor-still-reports
state: approved
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Doctor --fix refuses under an inherited verification context, and read-only doctor still reports

## Intent

doctor --fix refuses under an inherited verification context, and read-only doctor still reports

## Affected Canonical Specs

- `cli`

## Acceptance Criteria

- doctor --fix, with or without --dry-run or --close-late, refuses under an inherited SPECSYNC_VERIFICATION_CONTEXT with the one recursion diagnostic and exit 1; doctor without --fix gives the same output and exit status with the marker as without it (red then green)

## No-spec Rationale

Not applicable
