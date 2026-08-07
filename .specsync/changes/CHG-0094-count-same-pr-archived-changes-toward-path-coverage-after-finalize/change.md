---
id: CHG-0094-count-same-pr-archived-changes-toward-path-coverage-after-finalize
state: approved
type: feature
base_commit: d6eb2fa43c0fa7f1a1703e994cf2286bd2679120
---

# Count same-PR archived changes toward path coverage after finalize

## Intent

count same-PR archived changes toward path coverage after finalize

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- audit and CI path coverage treat archived changes present in the current PR delivery as covering their affected_paths so same-PR finalize archive tips do not fail Lifecycle gate with zero actives

## No-spec Rationale

After change ship on the same PR, Lifecycle gate fails path coverage because zero actives remain while product paths are still in the delivery diff. Archived packages in that diff must still cover their affected_paths.
