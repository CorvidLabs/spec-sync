---
id: bump-cap-primitives-to-4-0-3-for-rustsec-2026-0482
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Bump cap-primitives to 4.0.3 for RUSTSEC-2026-0482

## Intent

bump cap-primitives to 4.0.3 for RUSTSEC-2026-0482

## Affected Canonical Specs

- None

## Acceptance Criteria

- cargo audit no longer reports RUSTSEC-2026-0482; the full suite passes unchanged

## No-spec Rationale

dependency security patch; no behavior or API change
