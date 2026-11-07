---
change: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history
artifact: design
---

# Design

- `ledger_succession`: compare approvals with `relayed_by` cleared on each, reopen events with `relayed_by` cleared on their `superseded_approval`, and the superseded approval with the earlier terminal approval the same way. A small helper projects the JSON values; every other key is compared as before.
- The definition digest hashes the correction ledger through `correction_digest_projection`, the projection both correction digests use. The historical definition digest, which authenticates a committed correction as a trusted anchor, hashes it the same way; otherwise a relayed correction would stop being a trusted anchor.
- `validate_trusted_correction_history` compares `correction_digest_projection` of each trusted correction with that of the current one.
- No new public API; the three call sites reuse the existing projection.
