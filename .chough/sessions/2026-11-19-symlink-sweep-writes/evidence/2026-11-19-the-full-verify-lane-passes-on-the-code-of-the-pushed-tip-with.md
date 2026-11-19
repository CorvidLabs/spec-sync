---
kind: evidence
id: 2026-11-19-the-full-verify-lane-passes-on-the-code-of-the-pushed-tip-with
aliases: []
claim: "The full verify lane passes on the code of the pushed tip with both changes: unit suite 3095 passed, 0 failed, 2 ignored; integration suite 648 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the six gate scripts."
result: passed
evidence_kind: product-test
method: fledge lanes run verify with an isolated git config (GIT_CONFIG_GLOBAL), rustc 1.89.0, 10 steps in 34 minutes on a loaded machine
revision: 3a18f83c7701b31854549fa595f5a32c5c6c4302
content: 832a247c578e6ffff491fae9f2cbe21f1e457b4c792e5da8b4254ec7becd3b64
environment: local-macos-arm64
criteria: [SETUP-6, CHECK-1.e]
requirements: []
contracts: []
limitations:
  - "macOS arm64 only for this entry; the Linux arm64 container run is recorded in the earlier observation. No hosted CI: the private fork runs no Actions. With this machine's own global git config, 4 changelog unit tests fail under tag.gpgsign."
observed: 2026-11-19T05:53:55Z
actor:
  - agent: claude-code
    terminal: t5r
---
**The full verify lane passes on the code of the pushed tip with both changes: unit suite 3095 passed, 0 failed, 2 ignored; integration suite 648 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the six gate scripts.**

Recorded as passed in local-macos-arm64 at revision 3a18f83c7701, observed 2026-11-19T05:53:55Z.
