---
kind: evidence
id: 2026-11-07-revised-change-only-candidate
aliases: [OBS-product-test-e6cf3467399f08e8e24dd078]
claim: "Revised change-only candidate 4d245b9b15b46ebe9494932d03d0f2918707688c passed all 8 declared verify steps in 20m13s (t1bn exit0), with identical start/end SHA and 0-byte before/after manifests: 2903 unit+528 integration passed, 2 benchmarks ignored; fmt/lint/types, release build, 62 strict specs zero warnings and all workflow validators passed. Rust1.89/Python3.14, jobs2/testthreads4/nice10."
result: passed
evidence_kind: product-test
method: automated
revision: 4d245b9b15b46ebe9494932d03d0f2918707688c
content: 611014ce6ac352c000943f818740fdd91710ef7e93ce50402e21a723372ca2d1
environment: local-macos-arm64
fingerprint: sha256:f76024ae79d08b48ac38118fb536206c1f98f1ddf12a516c4991ae1d3bd5b35b
criteria: [SHIP-3.c]
requirements: [REQ-change-142]
contracts: [specs/change/change.spec.md]
limitations:
  - Local macOS only. Hosted CI, soft provenance result and fresh human implementation review remain distinct. Old e93f247b full pass is historical.
observed: 2026-11-07T07:52:40Z
---
**Revised change-only candidate 4d245b9b15b46ebe9494932d03d0f2918707688c passed all 8 declared verify steps in 20m13s (t1bn exit0), with identical start/end SHA and 0-byte before/after manifests: 2903 unit+528 integration passed, 2 benchmarks ignored; fmt/lint/types, release build, 62 strict specs zero warnings and all workflow validators passed. Rust1.89/Python3.14, jobs2/testthreads4/nice10.**

Recorded as passed in local-macos-arm64 at revision 4d245b9b15b4, observed 2026-11-07T07:52:40Z.
