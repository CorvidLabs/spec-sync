---
kind: evidence
id: 2026-11-20-657-red-without-the-code-at-e48d22fe
aliases: []
claim: "Red first at e48d22fe, the tests alone (the code set aside): 11 compile errors, in_flight_coverer, TerminalEvidenceValidity::ProvisionallyCovered and ChangeSummary.provisional_coverage do not exist. The ported test also failed at runtime on main a28e3042: the accepted change read stale, not provisionally-covered. cargo test --bin specsync through one-lane, the neutral git config as a prefix only (#1092)."
result: failed
evidence_kind: product-test
revision: c7e5daba2b794e31975863df7ed90d1f2c152e01
content: d94588e6d0f6595361b1100b1fa6c55ef793d3486e3a31e0f12e5c5ddc8c9173
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:ed8a3f1c5d043719806a3c143e626ff4b521219f3e668a60ac47882101ce8722
external: ext-df19e3
criteria: [SHIP-13, SHIP-13.a, SHIP-13.b, SHIP-13.c, SHIP-13.d]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T07:30:03Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red first at e48d22fe, the tests alone (the code set aside): 11 compile errors, in_flight_coverer, TerminalEvidenceValidity::ProvisionallyCovered and ChangeSummary.provisional_coverage do not exist. The ported test also failed at runtime on main a28e3042: the accepted change read stale, not provisionally-covered. cargo test --bin specsync through one-lane, the neutral git config as a prefix only (#1092).**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision c7e5daba2b79, observed 2026-11-20T07:30:03Z.
