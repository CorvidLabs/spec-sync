---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: requirements
---

# Requirements

Legacy acceptance-manifest reconstruction SHALL assign the exact delivery owner to
production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers
validate without per-repo remediation.

- Only pre-manifest (legacy) reconstruction is relaxed; current acceptance stays fail-closed.
- Historical aggregate reproduction, closing-approval authentication, and the exactly-one
  distinct reconstruction rule are unchanged.
- The exact delivery owner assignment appears only in reconstructed manifests, never in newly
  signed ones.
- No new command, state transition, or persisted evidence format is introduced.
