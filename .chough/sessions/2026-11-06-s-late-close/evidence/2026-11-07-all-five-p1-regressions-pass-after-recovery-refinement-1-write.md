---
kind: evidence
id: 2026-11-07-all-five-p1-regressions-pass-after-recovery-refinement-1-write
aliases: [OBS-product-test-a44413a32a903e5b908007e8]
claim: "All five P1 regressions pass after recovery refinement: 1 write-boundary unit matrix plus 4 CLI tests. The matrix now follows the final-component refusal by removing only the symlink, then closes both plans byte-for-byte with verified false and unchanged outside bytes. Linked/dangling parent dry-run/write refusal and parent recovery remain green."
result: passed
evidence_kind: product-test
revision: 8ad024f86f52572c95c679f801a648f199b53a9b
content: f212182c825c7091fdf8d90b9733a880b3b8886def7c3a40bad380e674620c39
environment: local-macos-arm64
fingerprint: sha256:f31f29b3ebff971ba1bc8975396490d80243dfcef9c2521d750559b5eb6a6723
criteria: [SHIP-12]
requirements: [REQ-change-143]
contracts: [specs/change/change.spec.md]
limitations:
  - Focused checks; final full verify and hosted CI remain pending. No arbitrary external filesystem-writer race or Windows qualification.
observed: 2026-11-07T06:58:49Z
---
**All five P1 regressions pass after recovery refinement: 1 write-boundary unit matrix plus 4 CLI tests. The matrix now follows the final-component refusal by removing only the symlink, then closes both plans byte-for-byte with verified false and unchanged outside bytes. Linked/dangling parent dry-run/write refusal and parent recovery remain green.**

Recorded as passed in local-macos-arm64 at revision 8ad024f86f52, observed 2026-11-07T06:58:49Z.
