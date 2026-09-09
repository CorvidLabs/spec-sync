---
id: correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes
state: verifying
type: bug_fix
base_commit: d0fb1621577ee3fc7f7d1a4e39f55ebaa6ded7fe
---

# Correct the 6.0 init version stamp, quickstart example, and release notes

## Intent

Correct the 6.0 init version stamp, quickstart example, and release notes

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A fresh specsync init writes 6.0.0 to .specsync/version; examples/quickstart passes specsync check --strict --require-coverage 100 from its own root and reports an added undocumented export; the [6.0.0] release notes no longer claim check prints an active-change count or that a post-merge binding job exists

## No-spec Rationale

No canonical spec contract changes: SDD_VERSION is already documented in change.spec.md, the cmd_init edits touch only its requirements and testing companions, and the rest is examples, tests, and documentation
