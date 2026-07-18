---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: context
---

# Context

Trust 1.0.1 dogfooding reopened a legacy accepted change after release metadata moved. Fresh
SpecSync 5.1 acceptance correctly required a per-input manifest, then rejected
`scripts/validate.py`: the historical change named `trust-action`, while the current canonical
file owner is `trust-policy`. Editing `affected_specs` would broaden an already-applied semantic
definition, and the existing acceptance guard correctly rejects that path. The supported
`change correct` transition covers only accepted interview booleans and cannot express exact
input ownership.

The missing operation is not a semantic rescope. It is a narrowly audited statement that one
already-scoped input also has a canonical owner used only for acceptance-manifest construction.
The original affected specs, deltas, approvals, reopen event, and prior verification remain
unchanged. A current definition approval, verification, and closing approval are still required.

Strict validation after CHG-0043 acceptance exposed a coupled manifest defect: its protected
legacy baseline ledger was explicitly in scope but was discarded by the generic dated-archive
volatility filter. This change also preserves that one protected ledger path so the accepted
baseline authority can sign the evidence its integrity validator requires.
