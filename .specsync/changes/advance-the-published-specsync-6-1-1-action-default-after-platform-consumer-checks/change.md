---
id: advance-the-published-specsync-6-1-1-action-default-after-platform-consumer-checks
state: implementing
type: bug_fix
base_commit: e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa
---

# Advance the published SpecSync 6.1.1 Action default after platform consumer checks

## Intent

Advance the published SpecSync 6.1.1 Action default after platform consumer checks

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- After actual immutable-candidate Ubuntu/macOS qualification, validated stable v6.1.1 assets and successful immutable-A explicit-version Linux/macOS consumers, advance the separate main Action wrapper default, offline declaration and corresponding docs to published 6.1.1. Preserve the composite runtime script bytes and explicit CI/Trust candidate mirrors.
- Preserve immutable v6.1.1 and RC refs at qualified candidate e8fd149d4d9e7b8df9ea1d40d446df67da13f2fa with embedded default 6.1.0 and explicit version 6.1.1 guidance; preserve the immutable v6.0.0 caveat. Represent the exact accepted Purpose with one normal github SpecSection delta, unchanged permanent REQ-github-002, actual current-parent canonical version and actual new change ID, preserving A and all historical rows.
- Remove only the obsolete prepared-release/published-version sentence from site/src/content/docs/configuration.md while preserving preceding reader/downgrade warnings and without adding a new runtime or distribution claim. Qualify the fresh exact B tree and normal native/CI/review/ship gates. Floating v6 may move only after omitted-input Linux/macOS consumers pass on the exact selected B target, without version override or immutable-tag movement.

## No-spec Rationale

Not applicable
