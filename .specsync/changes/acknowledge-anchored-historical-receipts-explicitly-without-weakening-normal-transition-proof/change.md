---
id: acknowledge-anchored-historical-receipts-explicitly-without-weakening-normal-transition-proof
state: implementing
type: feature
base_commit: 0b06c18aa2b6d6549f0bdb1e221b7a17e284c099
---

# Acknowledge anchored historical receipts explicitly without weakening normal transition proof

## Intent

Acknowledge anchored historical receipts explicitly without weakening normal transition proof

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- Opt-in historical reconciliation preserves authenticated committed predecessor receipts, labels old transitions unproven, remains bound to exact path/module/digest obligations, and requires newly approved scope plus fresh current verification and scoped review before closure. Normal before/after succession stays strict.

## No-spec Rationale

Not applicable
