---
id: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
state: implementing
type: refactor
base_commit: f3a65dc5577b0bc15d4ef45b796b3e7c4e81c44d
---

# Sever specsync check and comment from the trust layer: lifecycle state becomes informational and never affects exit status

## Intent

Sever specsync check and comment from the trust layer: lifecycle state becomes informational and never affects exit status

## Affected Canonical Specs

- `cmd_check`
- `cmd_comment`
- `change`

## Acceptance Criteria

- specsync check SHALL NOT exit non-zero because of SDD lifecycle state; it SHALL report the active-change count and emit lifecycle findings as stderr warnings, with exit status determined solely by spec validation, enforcement mode, --strict, and --require-coverage; specsync comment SHALL report spec-check results only; the orphaned quiet-output SDD check path SHALL be removed rather than left dead; and the integration tests asserting the removed behavior SHALL be deleted rather than adapted.

## No-spec Rationale

Not applicable
