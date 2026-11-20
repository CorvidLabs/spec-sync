---
kind: evidence
id: 2026-11-20-532-reproduced-through-the-binary
aliases: []
claim: "Reproduction (evidence/532-repro/run.sh): two real clones of one origin approve the same change, A merges B with git merge -X ours; on released 6.3.0 and on #428's head 9b75e8ba, change status, change check and change audit --strict exit 0 while clone B's approval is in history and not at HEAD."
result: failed
evidence_kind: product-test
revision: b220fecd3a50161f7fb44ebdf71c510feb05732e
content: a6d5de1311e931a220d14019971ea4a2e8fb58919c9502cb148e887a009c9870
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:3b14e25952f71ab3ad904aecac95838146b226f987cb9be80470c5470e0cac2c
external: ext-f798fd
criteria: [CHANGE-6, CHANGE-6.a]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T16:07:55Z
actor:
  - agent: claude-code
    terminal: t59
---
**Reproduction (evidence/532-repro/run.sh): two real clones of one origin approve the same change, A merges B with git merge -X ours; on released 6.3.0 and on #428's head 9b75e8ba, change status, change check and change audit --strict exit 0 while clone B's approval is in history and not at HEAD.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision b220fecd3a50, observed 2026-11-20T16:07:55Z.
