---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: research
---

# Research

- #1021 (filed from the CorvidLabs/site recovery): archived retired records whose custodian
  drifted make every roster-loading command fail with `retirement replacement has stale or
  unauthenticated evidence`, including the `change reopen` the error prescribes. Quarantining
  the retired records is refused as tampering. Deadlock.
- Reproduced on the live site repository (branch `custody-repair-test`): `change status` of
  the retired `bring-corvidlabs-xyz-...` record and `change audit` both fail. On current
  main the first failure is `retirement cannot transfer signed owners of .agents`: the
  receipt (written before the #1018 `@exact:*` byte-equality rule) recorded a
  `same-signed-owners` classification for a non-file `@exact:delivery` entry, and
  validation-time re-derivation applies today's stricter rule to the historical receipt.
- The custodian's drift itself (22 modified inputs, 2 recorded-missing) is legitimate
  post-acceptance content change; the same drift is reported as `stale` terminal evidence on
  the custodian's own record, as data, without failing the load.
- `retirement_custody` serves two masters today: retire-time writing (needs freshness and
  current-rule classification) and validation (needs neither). Only the retire-time caller
  keeps them.
