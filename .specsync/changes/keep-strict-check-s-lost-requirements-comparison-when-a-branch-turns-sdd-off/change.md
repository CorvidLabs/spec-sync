---
id: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
state: draft
type: bug_fix
base_commit: 2e2ddab1af06a6b8e877a3efd6fb40801786a2fc
---

# Keep strict check's lost-requirements comparison when a branch turns SDD off

## Intent

Keep strict check's lost-requirements comparison when a branch turns SDD off

## Affected Canonical Specs

- `cmd_check`
- `change`

## Acceptance Criteria

- On a branch whose trusted comparison base has SDD enabled, `check --strict` refuses an unexplained loss of a canonical requirement and names it when the branch sets `enabled` to false or deletes `.specsync/sdd.json`, committed or not, as `change audit --strict` does. A project whose comparison base never enabled SDD, or has it disabled there, keeps drift-only strict check.

## No-spec Rationale

Not applicable
