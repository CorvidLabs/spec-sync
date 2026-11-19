---
kind: evidence
id: 2026-11-19-1012-red-control-on-main-3bebe8b0
aliases: []
claim: "Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, merge_driver::tests::quoted_yaml_timestamps_stay_strings_after_a_union fails: the quoted 2026-11-07 loses its quotes in the merged list. This is the defect #1012 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: f8b341c574e0b61113983999a8dad19506f9eb24
content: 2f700bb9bf88dfae0d444c6fbdcd38ff8216f3e8fa75c8fb11d52fee2272fa99
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:a6dbba362272dce9d12d7c6cd82cb7a6ca16a0e1b600b11c8b281616eceb5c89
external: ext-b16d8d
criteria: [MERGE-5.a]
requirements: []
contracts: [specs/merge/merge.spec.md]
limitations: []
observed: 2026-11-19T08:47:15Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control: on main 3bebe8b0 with only the 11-13 failing-test commit applied, merge_driver::tests::quoted_yaml_timestamps_stay_strings_after_a_union fails: the quoted 2026-11-07 loses its quotes in the merged list. This is the defect #1012 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision f8b341c574e0, observed 2026-11-19T08:47:15Z.
