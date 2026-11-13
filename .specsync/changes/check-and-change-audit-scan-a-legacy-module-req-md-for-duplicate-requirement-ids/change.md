---
id: check-and-change-audit-scan-a-legacy-module-req-md-for-duplicate-requirement-ids
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Check and change audit scan a legacy module.req.md for duplicate requirement IDs

## Intent

check and change audit scan a legacy module.req.md for duplicate requirement IDs

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- check and change audit refuse a requirement ID repeated in a legacy <module>.req.md, naming the ID and the file, as they do in requirements.md (red then green)

## No-spec Rationale

Makes check and change audit match REQ-validator-045 and REQ-change-125, which refuse a requirements file that repeats a requirement ID: a legacy {module}.req.md companion is a requirements file, as the hash cache and the merge driver already read it, but neither scan opened it.
