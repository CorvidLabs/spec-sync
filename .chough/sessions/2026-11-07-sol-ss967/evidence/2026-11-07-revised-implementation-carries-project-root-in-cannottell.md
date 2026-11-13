---
kind: evidence
id: 2026-11-07-revised-implementation-carries-project-root-in-cannottell
aliases: [OBS-product-test-5d418cd517a127283b86d70b]
claim: "Revised implementation carries project_root in CannotTell, leaving command adapters unchanged: focused t1bh passed 9 unit/command and all 4 binary integration tests, including the full printed recovery from a quoted nested project parent. Its subsequent audit on dirty revert paths failed; audit must be rerun on the clean revised commit."
result: passed
evidence_kind: product-test
method: automated
revision: 4c0e5e48fce55fbf8ed9f5a2d2c4f585897bea79
content: 611014ce6ac352c000943f818740fdd91710ef7e93ce50402e21a723372ca2d1
environment: local-macos-arm64
fingerprint: sha256:2ba1e08e5dd36e984e9b20c0be2aa09526ab029d0deb1933dfc3d4da50d6d389
criteria: [SHIP-3.c]
requirements: [REQ-change-142]
contracts: [specs/change/change.spec.md]
limitations:
  - This observation covers product assertions only; it does not mark the failed dirty-tree audit passed or reuse the old e93f247b full lane as current.
observed: 2026-11-07T07:09:21Z
---
**Revised implementation carries project_root in CannotTell, leaving command adapters unchanged: focused t1bh passed 9 unit/command and all 4 binary integration tests, including the full printed recovery from a quoted nested project parent. Its subsequent audit on dirty revert paths failed; audit must be rerun on the clean revised commit.**

Recorded as passed in local-macos-arm64 at revision 4c0e5e48fce5, observed 2026-11-07T07:09:21Z.
