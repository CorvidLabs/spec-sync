---
kind: evidence
id: 2026-11-20-1108-correction-the-update-was-onto-3d57fa13
aliases: []
claim: "Correction to 2026-11-20-1108-update-onto-c14bdf69-lane-at-715c1068: that update merged main 3d57fa13 (the 6.4.0 release commit), not c14bdf69, and its lane ran there. After the archive (77887bea), main c14bdf69 (#1136: release docs, the Action default) was merged at 10cc4e32 with one CHANGELOG [Unreleased] conflict, both entries kept. It changes none of #1108's own lines. change audit --strict passes with no active change; CI runs on 10cc4e32."
result: passed
evidence_kind: product-test
revision: 10cc4e32d73233d940b7a045c010dc04aafe4836
content: b926bb61fb08a78a963ea02e0ce5dd37238f8710d842d6185162565a1653668a
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:554283c1d3d4d2e76f785f714f11499237199f827099b65033e86f040bdcc74f
external: ext-e57d11
criteria: [CHANGE-17]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T21:31:47Z
actor:
  - agent: claude-code
    terminal: t59
---
**Correction to 2026-11-20-1108-update-onto-c14bdf69-lane-at-715c1068: that update merged main 3d57fa13 (the 6.4.0 release commit), not c14bdf69, and its lane ran there. After the archive (77887bea), main c14bdf69 (#1136: release docs, the Action default) was merged at 10cc4e32 with one CHANGELOG [Unreleased] conflict, both entries kept. It changes none of #1108's own lines. change audit --strict passes with no active change; CI runs on 10cc4e32.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 10cc4e32d732, observed 2026-11-20T21:31:47Z.
