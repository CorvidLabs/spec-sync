---
id: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
state: implementing
type: feature
base_commit: 801639111f891ea34d01078a20b9d8ac20668a61
---

# Address PR 531 review by validating correction-ledger health before mutating lifecycle commands and incrementing the cmd_change contract version

## Intent

Address PR 531 review by validating correction-ledger health before mutating lifecycle commands and incrementing the cmd_change contract version

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- Mutating text commands reject an invalid correction ledger before persistence; read-only show, status, and list remain fail-closed; valid mutations retain existing behavior; the cmd_change contract version increments; focused regression and strict checks pass.

## No-spec Rationale

Not applicable
