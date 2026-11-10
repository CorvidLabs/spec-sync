---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: context
---

# Context

## Decisions

- Validation trusts the receipt's recorded classification as a historical fact pinned by
  committed history; it checks the classification is consistent with the receipt's own
  fields instead of re-deriving it from current evidence under current rules.
- Custody persistence is judged by path presence and unchanged signed owners in the
  custodian's current authenticated manifest. Byte/digest evolution under the same owners is
  the custodian's audited lifecycle, exactly as successor coverage already treats accepted
  evolution for ordinary changes.
- Custodian freshness against the live tree leaves retirement validation entirely; it is
  reported as terminal-evidence data on the custodian, where the reopen remedy is named.

## Ruled out

- Amending or re-signing receipts after custodian evolution: receipts are immutable and
  history-pinned by design.
- Accepting owner changes post-retirement: no byte-continuity proof exists across an
  evolution window, so owner changes still refuse.
- Chaining retirements (retiring a custodian): the replacement state rule already refuses.
