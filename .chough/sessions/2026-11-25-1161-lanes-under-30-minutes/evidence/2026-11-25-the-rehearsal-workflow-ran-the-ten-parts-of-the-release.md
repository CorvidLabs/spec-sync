---
kind: evidence
id: 2026-11-25-the-rehearsal-workflow-ran-the-ten-parts-of-the-release
aliases: []
claim: "The rehearsal workflow ran the ten parts of the release-candidate lane at a6c79b5a, each under its 30-minute limit, and the whole run took 22m23s. macOS: tests 1 of 4 18m32s, 2 of 4 21m37s, 3 of 4 22m11s, 4 of 4 21m22s, build 15m45s; all five macOS jobs started together. Ubuntu: 4m43s, 4m30s, 4m08s, 4m47s, build 5m51s. The slices-add-up job passed: macOS slices 1047, 1047, 1047 and 1046 are its whole list of 4187, and Ubuntu's 1049, 1049, 1048 and 1048 are its whole list of 4194. The macOS qualification of v6.4.2-rc.2 took 78m32s as one job."
result: passed
evidence_kind: product-test
revision: a29b4a439c9f19b5103d30b5c40e4c32c010d678
content: 253180fc8d09f289ae64634152d74aece54980851c7599e69f736ab6f8039bed
environment: "GitHub-hosted runners: macos-15 and ubuntu-latest, pull request #1168, rehearsal run 45542239113 at a6c79b5a"
fingerprint: sha256:32a0da0c54802bdaf2d7574aea36f1c5b80039c8e992d12c21ec76c7d0f9a9cc
external: ext-462a20
criteria: [SPEED-2.b.3]
requirements: []
contracts: []
limitations: []
observed: 2026-11-25T04:33:23Z
actor:
  - agent: claude-code
    terminal: t14v
---
**The rehearsal workflow ran the ten parts of the release-candidate lane at a6c79b5a, each under its 30-minute limit, and the whole run took 22m23s. macOS: tests 1 of 4 18m32s, 2 of 4 21m37s, 3 of 4 22m11s, 4 of 4 21m22s, build 15m45s; all five macOS jobs started together. Ubuntu: 4m43s, 4m30s, 4m08s, 4m47s, build 5m51s. The slices-add-up job passed: macOS slices 1047, 1047, 1047 and 1046 are its whole list of 4187, and Ubuntu's 1049, 1049, 1048 and 1048 are its whole list of 4194. The macOS qualification of v6.4.2-rc.2 took 78m32s as one job.**

Recorded as passed in GitHub-hosted runners: macos-15 and ubuntu-latest, pull request #1168, rehearsal run 45542239113 at a6c79b5a at revision a29b4a439c9f, observed 2026-11-25T04:33:23Z.
