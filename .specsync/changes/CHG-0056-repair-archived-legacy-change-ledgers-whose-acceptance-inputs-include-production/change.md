---
id: CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production
state: accepted
type: bug_fix
base_commit: 37120cb60407efed08b4868858e76fb847d1ee9d
---

# Repair archived legacy change ledgers whose acceptance inputs include production source with no canonical owner by resolving unowned production source to the exact delivery owner during legacy acceptance-manifest reconstruction, so adoption-era archived records validate under current rules without per-repo remediation

## Intent

Repair archived legacy change ledgers whose acceptance inputs include production source with no canonical owner by resolving unowned production source to the exact delivery owner during legacy acceptance-manifest reconstruction, so adoption-era archived records validate under current rules without per-repo remediation

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- An archived 5.0.1-era change whose acceptance inputs include production source with no canonical owner (the CorvidLabs/agent-findings CHG-0001 case) validates under archived historical-integrity checks with the unowned input assigned the exact delivery owner in the reconstructed manifest; current acceptance still rejects unowned production source fail-closed; regression tests cover both the relaxed legacy reconstruction and the unchanged strict current path; the exact-delivery assignment appears only in reconstructed legacy manifests and never in newly signed ones.

## No-spec Rationale

Not applicable
