---
id: parse-a-broken-policy-beside-a-baseline-and-retry-the-archive-commit
state: implementing
type: bug_fix
base_commit: 9f4b72dd5c728f129c47a74493051b84d9d475e2
---

# Parse a broken policy beside a baseline and retry the archive commit

## Intent

Parse a broken policy beside a baseline and retry the archive commit

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cmd_rules`

## Acceptance Criteria

- Opening a change fails when .specsync/sdd.json cannot be parsed, including when a workflow-v2 baseline exists.
- change finalize commits an archive that was left uncommitted, and its next action says to push that commit before merging.
- specsync rules does not treat an escaping principles path as a usable file.

## No-spec Rationale

Not applicable
