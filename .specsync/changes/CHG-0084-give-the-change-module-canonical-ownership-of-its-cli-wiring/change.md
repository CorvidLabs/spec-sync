---
id: CHG-0084-give-the-change-module-canonical-ownership-of-its-cli-wiring
state: implementing
type: bug_fix
base_commit: e59a5a575f40a9426777696dc5988ef3365843d2
---

# Give the change module canonical ownership of its CLI wiring

## Intent

Give the change module canonical ownership of its CLI wiring

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- specs/change/change.spec.md claims src/commands/change.rs, so finalize resolves canonical ownership for the change module's CLI wiring

## No-spec Rationale

Not applicable
