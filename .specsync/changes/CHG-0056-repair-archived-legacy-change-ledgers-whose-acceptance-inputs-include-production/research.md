---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: research
---

# Research

`resolved_acceptance_manifest` validates a signed manifest when one exists and only then falls
into `reconstruct_legacy_acceptance_manifest`, so the relaxed path is unreachable for any change
accepted under current rules. Inside `reconstruct_legacy_at_anchor` the historical aggregate is
reproduced first (`acceptance_input_digest`, which never consults owners), then
`acceptance_manifest` computes owners — the single point where
`acceptance input … is production source without deterministic canonical ownership` aborts
reconstruction for spec-less adoption repos.

`acceptance_input_owners` has exactly one production caller (inside `acceptance_manifest`), and
`acceptance_manifest` has three production callers: the current acceptance flow, the
succession-base digest helper, and legacy reconstruction. A policy parameter therefore reaches
the error site with two-line call-site changes and no behavior change anywhere else.

`EXACT_DELIVERY_OWNER` is already the deterministic owner for non-production delivery paths, and
owner-correction records reserve `@exact:` owners against manual assignment; assigning it inside
reconstruction keeps the reserved-owner rule intact because reconstructed manifests are computed,
never persisted or signed.
