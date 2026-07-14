---
id: CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2
state: verifying
type: feature
base_commit: 21e44ecf33f4fe876820ef3ef8f19553341da15a
---

# Stabilize SpecSync 5 lifecycle integrity and strict validation for 5.0.2

## Intent

Stabilize SpecSync 5 lifecycle integrity and strict validation for 5.0.2

## Affected Canonical Specs

- `change`
- `validator`

## Acceptance Criteria

- Strict validation detects unacknowledged numeric sequence collisions across active and archived workspaces and repository-backed sequence claims prevent parallel branches from silently reusing a number while historical accepted evidence remains immutable; recursive verification fails once without orphan processes; failed verification retries can succeed without weakening unrelated gates; approved canonical successors replace stale predecessors only for exactly governed modules; semantic deltas resolve registry-backed canonical paths; configured HTML and static sources count toward non-vacuous coverage; strict checking rejects unfilled companion scaffold markers with precise diagnostics; the focused and full validation suites plus SpecSync Trust Augur Attest and hosted platform checks pass

## No-spec Rationale

Not applicable
