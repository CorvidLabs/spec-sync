---
id: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
state: archived
type: bug_fix
base_commit: 49d2326a8f80232b8fee94a5073c4c1a66fd2057
---

# Keep retired custody valid across audited custodian drift so the recorded reopen remedy stays reachable

## Intent

Keep retired custody valid across audited custodian drift so the recorded reopen remedy stays reachable

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Reopening a drifted retirement custodian no longer aborts on the retired records' custody validation; after audited re-acceptance with evolved digests, custody still validates while owners hold; owner or path loss still refuses

## No-spec Rationale

Not applicable
