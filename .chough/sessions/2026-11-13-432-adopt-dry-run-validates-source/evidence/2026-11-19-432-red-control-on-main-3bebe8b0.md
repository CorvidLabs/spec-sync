---
kind: evidence
id: 2026-11-19-432-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails: change adopt --source bogus --dry-run exits 0 and prints an import plan. This is the defect #432 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: f8b341c574e0b61113983999a8dad19506f9eb24
content: 2f700bb9bf88dfae0d444c6fbdcd38ff8216f3e8fa75c8fb11d52fee2272fa99
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:8c2f548e6caf6a14658d1b898bb0b744ac3e52017991c8fb043f498675da8520
external: ext-c39ba3
criteria: [SETUP-5.b]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T08:47:14Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does fails: change adopt --source bogus --dry-run exits 0 and prints an import plan. This is the defect #432 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision f8b341c574e0, observed 2026-11-19T08:47:14Z.
