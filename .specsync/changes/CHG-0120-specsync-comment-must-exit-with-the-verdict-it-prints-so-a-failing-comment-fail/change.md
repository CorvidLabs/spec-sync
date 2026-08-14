---
id: CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail
state: implementing
type: bug_fix
base_commit: 6e007ac3fc26ee747eb47bd7d24e8e6e93153a47
---

# Specsync comment must exit with the verdict it prints, so a failing comment fails the CI step that posted it

## Intent

specsync comment must exit with the verdict it prints, so a failing comment fails the CI step that posted it

## Affected Canonical Specs

- `cmd_comment`

## Acceptance Criteria

- A project that fails validation makes 'specsync comment' exit non-zero, matching the exit code of 'specsync check' on the same project and matching the verdict rendered in the comment body it prints. A passing project still exits zero. '--require-coverage N' is honored by 'comment' exactly as it is by 'check', 'score', 'report' and 'deps': a tree below the threshold exits non-zero. The comment body itself is unchanged; only the process exit status is corrected.

## No-spec Rationale

Not applicable
