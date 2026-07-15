---
id: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
state: verifying
type: feature
base_commit: 7ea73117168c72086bfc43e020c37904e0b6d7c5
---

# Support audited append-only correction of accepted interview metadata without replaying canonical deltas

## Intent

Support audited append-only correction of accepted interview metadata without replaying canonical deltas

## Affected Canonical Specs

- `change`
- `cli_args`
- `cmd_change`

## Acceptance Criteria

- Accepted interview metadata cannot be overwritten in place; corrections append actor, non-empty reason, exact old and new values, timestamps, and portable digests while preserving the original record and approvals.
- Only explicitly supported interview fields can be corrected, and strict checking resolves the effective value through a validated append-only correction ledger.
- Corrections that change selected artifacts or policy scrutiny require complete artifacts plus fresh definition approval, verification, and closing approval.
- Canonical semantic deltas remain non-replaying and all prior evidence remains inspectable across portable checkouts and squash integration.
- CLI text and JSON surfaces expose original and corrected values, actor, reason, timestamps, digests, approval health, and deterministic next actions.
- Regression coverage includes repeated corrections, malformed ledgers, stale corrected evidence, portable checkout paths, squash integration, and rejection of unsupported fields.

## No-spec Rationale

Not applicable
