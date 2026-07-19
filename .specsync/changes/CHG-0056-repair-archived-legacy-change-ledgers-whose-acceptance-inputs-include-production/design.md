---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: design
---

# Design

Thread an explicit policy through manifest construction instead of weakening the check globally:
`acceptance_manifest` and `acceptance_input_owners` take an `UnownedProductionSource` mode with
`Reject` (default) and `AssignExactDelivery`. Only `reconstruct_legacy_at_anchor` passes
`AssignExactDelivery`; every current-flow caller (acceptance, succession-base digests, tests)
passes `Reject`, so the fail-closed posture for newly signed evidence is unchanged.

The relaxation is structurally bounded to genuine legacy records: any change accepted under
current rules carries a signed `acceptance_manifest`, so only pre-manifest (5.0.1-era) records
ever reach the relaxed path. The historical raw-content aggregate still must reproduce the signed
digest, the historical closing approval still must authenticate, and the reconstruction still must
be exactly one distinct result — the only change is that an unowned production-source input is
assigned `EXACT_DELIVERY_OWNER` in the reconstructed manifest instead of aborting it.

Alternatives considered (from the issue): an `archive-repair` command or archived-state
`correct-owner` — both impose per-repo, per-change, per-path correction loops (the same friction
as #398) on top of immutable history that cannot fail any other way; a blanket ownership
relaxation was rejected because current acceptance must keep forcing canonical spec coverage.
