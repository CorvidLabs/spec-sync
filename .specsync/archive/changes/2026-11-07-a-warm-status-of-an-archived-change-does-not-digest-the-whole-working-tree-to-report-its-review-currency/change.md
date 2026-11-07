---
id: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency
state: archived
type: feature
base_commit: 86857c8f63acc93b480ac441e32183b8b1ed3a29
---

# A warm status of an archived change does not digest the whole working tree to report its review currency

## Intent

A warm status of an archived change does not digest the whole working tree to report its review currency

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- warm_archived_status_git_processes_do_not_grow_with_tracked_files passes: 3000 more tracked files add fewer than 3 Git processes to a warm archived change status (committed failing first in a5b54557: 89 -> 196)
- an_archived_change_reports_the_scoped_review_currency_the_full_check_gives passes unchanged: an archived change reports the scoped_review_current the full check gives today, true right after finalize while the archive is uncommitted and false once committed, after later commits, after an unrelated edit and for a moved archive
- An archived change takes the project-input digest for scoped_review_current only while its archived package has uncommitted changes; a committed archive reports false without the digest
- States before archive compute scoped_review_current exactly as today; the next action, every gate and every refusal are unchanged
- On Raven (150 archives) a warm archived change status drops from 183 Git processes to about 53; before and after times are recorded

## No-spec Rationale

Not applicable
