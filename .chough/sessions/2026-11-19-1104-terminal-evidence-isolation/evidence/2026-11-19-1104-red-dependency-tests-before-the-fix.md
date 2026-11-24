---
kind: evidence
id: 2026-11-19-1104-red-dependency-tests-before-the-fix
aliases: []
claim: "The dependency tests of 06ac04fe, written after the fix, fail against the pre-fix sources: an accepted change with unchanged inputs and an unreadable state elsewhere does not read exact, and a failed candidate is not named in a successor search."
result: failed
evidence_kind: product-test
revision: 38b2dbfb2ffd1cc5db7ce186f15041c29be1601a
content: 4536f5d9e339c8c46fd9a45f49e7b89839ccc5fc3b324fbc5cba75dad01f9d5e
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:4562c1088b209c5f66c7788d86c416fab1dba44fe694a507b91946b71ccb9532
external: ext-811939
criteria: [CHANGE-7]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T20:15:38Z
actor:
  - agent: claude-code
    terminal: t59
---
**The dependency tests of 06ac04fe, written after the fix, fail against the pre-fix sources: an accepted change with unchanged inputs and an unreadable state elsewhere does not read exact, and a failed candidate is not named in a successor search.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 38b2dbfb2ffd, observed 2026-11-19T20:15:38Z.
