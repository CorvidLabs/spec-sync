---
id: keep-accepted-interview-correction-fail-closed-on-broken-change-records
state: archived
type: bug_fix
base_commit: 16a43b93a83a9487dacc38109af993e3d38cd8a1
---

# Keep accepted interview correction fail-closed on broken change records

## Intent

Keep accepted interview correction fail-closed on broken change records

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- When an unrelated change has corrupt committed workflow history, correcting accepted interview metadata must fail with that history error before writing any new evidence. The same correction must succeed with a valid project roster, and the healthy change’s read-only status must remain independent from that sibling.

## No-spec Rationale

Not applicable
