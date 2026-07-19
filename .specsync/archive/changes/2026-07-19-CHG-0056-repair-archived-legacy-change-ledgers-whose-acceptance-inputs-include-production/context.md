---
change: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
artifact: context
---

# Context

GitHub issue #397. Adoption-era (5.0.1) archived changes predate signed acceptance manifests, so
archived-integrity validation rebuilds a manifest through `reconstruct_legacy_at_anchor`. Owner
resolution rejects any production-source input with no canonical owner (`acceptance input … is
production source without deterministic canonical ownership`), which fails every reconstruction
(`found 0`) and leaves the contract gate permanently red for spec-less adoption repos — the real
case is CorvidLabs/agent-findings archived CHG-0001 covering `scripts/evaluate-agents.ts`.

No remediation exists: `change reopen` refuses archived changes, and `change correct-owner`
requires state `verifying` plus a canonical spec that lists the file — impossible with zero specs.

The signed raw-content aggregate still authenticates the historical bytes; only owner routing
metadata is underivable. Archived changes are immutable and no longer gate delivery, so the
deterministic exact-delivery owner used for non-production paths is the truthful routing for
unowned adoption-era inputs, with no per-repo repair burden.
