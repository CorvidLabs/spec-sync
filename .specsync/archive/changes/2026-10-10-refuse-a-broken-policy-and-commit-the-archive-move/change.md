---
id: refuse-a-broken-policy-and-commit-the-archive-move
state: archived
type: bug_fix
base_commit: 4d42cd401623481ae6b99e12142b7aa8767fc25e
---

# Refuse a broken policy and commit the archive move

## Intent

Refuse a broken policy and commit the archive move

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cmd_rules`

## Acceptance Criteria

- Opening a change and asking its status fail when .specsync/sdd.json cannot be parsed.
- specsync rules warns when the named principles file is not on disk.
- change finalize commits the archive move.

## No-spec Rationale

Not applicable
