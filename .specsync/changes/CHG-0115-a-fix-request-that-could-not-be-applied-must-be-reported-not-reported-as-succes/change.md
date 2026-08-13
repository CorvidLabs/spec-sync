---
id: CHG-0115-a-fix-request-that-could-not-be-applied-must-be-reported-not-reported-as-succes
state: implementing
type: bug_fix
base_commit: 12cbd50f5bb24b36da2dc56fe17dcf98743ac7da
---

# A fix request that could not be applied must be reported, not reported as success

## Intent

A fix request that could not be applied must be reported, not reported as success

## Affected Canonical Specs

- `cmd_check`

## Acceptance Criteria

- Running `specsync check --fix` against a spec file that cannot be written reports the path and the underlying error and exits non-zero, instead of exiting zero with no indication that nothing was written. A spec that cannot be read is reported the same way rather than skipped silently. A writable spec is still repaired and still exits zero, and `--fix --dry-run` against an unwritable spec still exits zero because it attempts no write.

## No-spec Rationale

Not applicable
