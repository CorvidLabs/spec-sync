---
id: CHG-0149-an-archived-change-package-must-not-leave-an-untrackable-husk
state: archived
type: feature
base_commit: 200add8f74ea826f271db0fa3432db0d68aec5e9
---

# An archived change package must not leave an untrackable husk

## Intent

an archived change package must not leave an untrackable husk

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- ship of a change whose deltas/ is empty leaves no untrackable empty directory in the dated archive package, so a checkout of a pre-archive commit removes the package entirely; a stray archive directory that contains no regular file at any depth is skipped by change new, change audit, change adopt and check instead of hard-failing with a raw OS error; a stray archive directory that does contain files but no state.json still hard-fails, so the tolerance cannot be satisfied by ignoring corruption.

## No-spec Rationale

Not applicable
