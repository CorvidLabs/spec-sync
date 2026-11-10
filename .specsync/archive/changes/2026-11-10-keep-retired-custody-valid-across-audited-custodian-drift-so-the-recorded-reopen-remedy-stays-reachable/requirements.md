---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: requirements
---

# Requirements

- When a retired change's custodian drifts after acceptance, loading the roster, `change
  status`, `change audit` and `change reopen` still work; the staleness is reported on the
  custodian's own record with the reopen remedy, which is then reachable.
- After the custodian is reopened and re-accepted through the audited lifecycle, retirement
  custody validates against the new evidence while every custody path remains under its
  recorded signed owners.
- A custody path missing from the custodian's current authenticated manifest, or carried
  under different owners, still refuses validation.
- Committed receipts whose recorded classifications predate newer classification rules keep
  validating; classification rules bind at retire time only.
- Retire-time requirements are unchanged: a fresh, independently accepted custodian with
  complete custody, refusing stale, unknown, tampered or circular input atomically.
