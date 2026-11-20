---
kind: evidence
id: 2026-11-20-640-red-candidate-anchors-at-9f8cea7e
aliases: []
claim: "Red first at 9f8cea7e: with only test-validate-release-candidate.py's 19 anchors moved to 6.4.0, the test fails 7 tests, every one an anchor the unbumped surfaces do not have yet (Validated release version 6.4.0, ci.yml and trust.yml candidate versions, Cargo.lock, site Action refs). Run with python3 through one-lane, the neutral git config as a prefix only (#1092)."
result: failed
evidence_kind: product-test
revision: 7ff79527093faf02d6d9d9370c673502f56e8501
content: 016054af00bb40e433cbc1ee4239b79fe0d437d5b8f2b4fd5bd7ed00a25b44df
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:1690f70769b507a06d298953e6732f17ec942e8162cea1702ce95cd80d6a82cd
external: ext-346783
criteria: [CI-1.a]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T01:39:20Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red first at 9f8cea7e: with only test-validate-release-candidate.py's 19 anchors moved to 6.4.0, the test fails 7 tests, every one an anchor the unbumped surfaces do not have yet (Validated release version 6.4.0, ci.yml and trust.yml candidate versions, Cargo.lock, site Action refs). Run with python3 through one-lane, the neutral git config as a prefix only (#1092).**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 7ff79527093f, observed 2026-11-20T01:39:20Z.
