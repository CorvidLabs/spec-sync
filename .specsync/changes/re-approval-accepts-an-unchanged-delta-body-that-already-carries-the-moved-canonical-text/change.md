---
id: re-approval-accepts-an-unchanged-delta-body-that-already-carries-the-moved-canonical-text
state: implementing
type: bug_fix
base_commit: 6ef61e6aae6d63b2586d0f9f216c47803dba4d9f
---

# Re-approval accepts an unchanged delta body that already carries the moved canonical text

## Intent

re-approval accepts an unchanged delta body that already carries the moved canonical text

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- re-approving an unchanged MODIFIED or REMOVED delta whose canonical block moved since approval succeeds and records the new base when the approval-time base text, recovered from git by its recorded digest, shows in a line-level three-way comparison that the body already carries every edit the current text made; otherwise, including when the base cannot be recovered, it is refused as before and nothing is recorded; no new flags

## No-spec Rationale

REQ-change-122 says a rebased and re-approved delta materializes keeping the other change's text it now carries; a body rebased before the other change landed could not be re-approved without a byte edit. Leif chose the three-way fix on 2026-11-10
