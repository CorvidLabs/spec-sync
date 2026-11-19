---
kind: evidence
id: 2026-11-19-1091-red-lock-create-at-e04f81bd
aliases: []
claim: "Red-first for Codex's round-2 P2 on #1100 (reproduced by corvid-agent at 57c36d84): at e04f81bd (the test alone) a_lock_the_approval_cannot_create_is_refused_in_its_words fails: with no lock file and .specsync mode 0555 the dry run exits 0 (would_approve) while the real approve fails 'failed to open lifecycle lock …: Permission denied'. Fixed in 036735e3."
result: failed
evidence_kind: product-test
revision: a768326cb740bf0b18695a809e780a09ad148436
content: e64ae66cbd71d774288cfe1a5df5c9c64e3ea40bb8a72ace702b222ff4e35925
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:7ad5edcdba39d79c7392030cb3075e8da39dfe94ef89a1cabd84d1dbe3e5d1c0
external: ext-d7f33e
criteria: [CHANGE-15.a]
requirements: [REQ-change-163]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T16:57:24Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first for Codex's round-2 P2 on #1100 (reproduced by corvid-agent at 57c36d84): at e04f81bd (the test alone) a_lock_the_approval_cannot_create_is_refused_in_its_words fails: with no lock file and .specsync mode 0555 the dry run exits 0 (would_approve) while the real approve fails 'failed to open lifecycle lock …: Permission denied'. Fixed in 036735e3.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision a768326cb740, observed 2026-11-19T16:57:24Z.
