---
kind: evidence
id: 2026-11-19-1091-red-fix-field-at-6d6ac9ea
aliases: []
claim: "Red-first after the send-back: at 6d6ac9ea (the tests alone) four dry-run tests fail. A definition refusal for an open question and an ownership refusal carry no fix; a .specsync/change.lock that is a directory passes the dry run though approve cannot open it (Codex P2); an ownership message's line breaks spill the text output over several lines (Codex P2). The fifth new test (no fix elsewhere) passes there, as it should. Fixed in 4d5d0440."
result: failed
evidence_kind: product-test
revision: 272abff42a0cf61efe11e38c5978037d20838b86
content: 05080872cdf9122086d1c065fc68da70ae8eae3a13bb3904a1ce15db9160df59
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:af872ebed7d6e64ee5d3cbe42f5c43268029b841cc71419ace9a56da3fe3ab6f
external: ext-6d3623
criteria: [CHANGE-15.d]
requirements: [REQ-cmd-change-040]
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-19T15:33:02Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first after the send-back: at 6d6ac9ea (the tests alone) four dry-run tests fail. A definition refusal for an open question and an ownership refusal carry no fix; a .specsync/change.lock that is a directory passes the dry run though approve cannot open it (Codex P2); an ownership message's line breaks spill the text output over several lines (Codex P2). The fifth new test (no fix elsewhere) passes there, as it should. Fixed in 4d5d0440.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 272abff42a0c, observed 2026-11-19T15:33:02Z.
