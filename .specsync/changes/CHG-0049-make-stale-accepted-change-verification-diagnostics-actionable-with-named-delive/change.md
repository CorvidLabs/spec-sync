---
id: CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive
state: verifying
type: feature
base_commit: 9bdd0beacbc40a610a2da590c05d8e41abc40904
---

# Make stale accepted-change verification diagnostics actionable with named delivery inputs and remediation

## Intent

Make stale accepted-change verification diagnostics actionable with named delivery inputs and remediation

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- When an accepted change's covered delivery inputs change after acceptance, 'specsync check' reports a stale-verification error that names the offending delivery input path and its owner, distinguishes 'no accepted successor covers this input' from 'a covering successor exists but its own evidence is stale' (naming those successor change IDs), and states the remediation (verify/accept the covering successor, or run 'specsync change reopen <id>'); all messages are deterministic with sorted successor IDs and no timestamps

## No-spec Rationale

Not applicable
