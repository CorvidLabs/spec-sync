---
id: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
state: implementing
type: feature
base_commit: ba890fc8b51a76e6eb0112c9150a4732a63ea23d
---

# Allow squash-accepted evidence on descendant branches

## Intent

Allow squash-accepted evidence on descendant branches

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Accepted evidence from a squash-merged change remains valid on a descendant feature branch when the current definition
- delivery inputs
- closing approval
- and committed accepted-state history all match; arbitrary off-history evidence still fails; regression and strict verification pass.

## No-spec Rationale

Not applicable
