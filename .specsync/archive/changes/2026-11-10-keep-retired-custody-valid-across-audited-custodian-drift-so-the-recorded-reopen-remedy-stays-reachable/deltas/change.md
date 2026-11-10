---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: delta
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-recovery-retirement-001

An obsolete audited-reopened obligation SHALL have an explicit terminal nonacceptance disposition with authenticated original evidence custody and an independently accepted current replacement; it SHALL grant no implementation, dependency or semantic succession credit.

Acceptance Criteria
- Retiring refuses atomically unless every original signed input is bound to retained evidence and a compatible replacement input in the custodian's fresh, authenticated, current evidence; missing, stale, unknown, tampered or circular custody refuses; original criteria, failures, approvals and canonical bytes remain unchanged; old readers fail closed instead of silently certifying retirement.
- Validation of a recorded retirement authenticates the custodian's recorded accepted evidence and checks that custody still holds: every custody entry names a frozen prior-manifest input at the recorded predecessor digest and owners, its recorded classification is consistent with the receipt's own fields, and the custodian's current authenticated manifest still carries the path under the recorded replacement owners. It does not re-run the custodian's live delivery-input freshness: custodian staleness is the custodian's own reported lifecycle state, so the reopen remedy it names stays reachable.
- Audited custodian evolution keeps custody valid while every custody path remains under its recorded signed owners; byte and manifest digest changes from a genuine re-acceptance are not a custody break. A custody path missing from the custodian's current authenticated manifest, or carried under different owners, refuses validation.
- A committed receipt's recorded classification is a historical fact; validation never re-derives it from current evidence under newer rules, so receipts written before a classification rule change keep validating.
