---
id: import-from-dir-review-follow-ups
state: archived
type: bug_fix
base_commit: 82334a70662b3a74b85a682af9792dcdcc6b4827
---

# Import --from-dir review follow-ups

## Intent

import --from-dir review follow-ups

## Affected Canonical Specs

- `importer`
- `cmd_import`

## Acceptance Criteria

- section splitting ignores ## lines inside fenced code blocks; depends_on, db_tables, implements and any H1 lead-in survive import; imported specs always render status: draft regardless of source status; a frontmatter-less markdown doc imports as a draft with a loud empty-files warning instead of being refused

## No-spec Rationale

Not applicable
