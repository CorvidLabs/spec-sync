---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-tests
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new tests, four literal-guidance tests fail (leading-hyphen id, control byte in a directory name, package probe as one literal path, glob characters in the restore). These are the defects #929, #930 and #931 report, fixed by this change; not new bugs."
result: failed
evidence_kind: product-test
revision: ea28c3e994c34dcd891f1a3a2e7f81a3dbc81088
content: bae28eaa71953f4e2bb8a213fdd76664c471cff9015f17f23be653111e7ce08f
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:a61cebded633ba622adddbb42fa2a567c8c9d6bcc216c1a22a80886f5558d435
external: ext-30a356
criteria: [CHANGE-7.a.2, CHANGE-7.b]
requirements: [REQ-change-093]
contracts: []
limitations: []
observed: 2026-11-13T19:22:03Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new tests, four literal-guidance tests fail (leading-hyphen id, control byte in a directory name, package probe as one literal path, glob characters in the restore). These are the defects #929, #930 and #931 report, fixed by this change; not new bugs.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision ea28c3e994c3, observed 2026-11-13T19:22:03Z.
