---
kind: evidence
id: 2026-11-20-688-red-through-the-binary-at-42a0210a
aliases: []
claim: "The tester's P-688-refusals-name-each-other through the binary, red first at 42a0210a: on a version-1 project, a later accepted change that edits the earlier change's src/a.rs without declaring it is not named by the earlier change's archive refusal (only the reopen is named); the later change's refusal names the earlier one. The guard test (a hand edit, no later change) passes. cargo test --test integration through one-lane, the neutral git config as a prefix only (#1092)."
result: failed
evidence_kind: product-test
revision: 6ec5a3f5f919a1a3fb20040bc38694e7ea655f6c
content: 7646c1ab311716d7857eea7be05d547381f01cbd570b2bd3980c9dffc2c53a1d
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:a8428920ee528b5f7a77cba46153fcb3c8e0536553596526dd18e155cb7e76a3
external: ext-758b9e
criteria: [SHIP-13.d]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T08:40:17Z
actor:
  - agent: claude-code
    terminal: t59
---
**The tester's P-688-refusals-name-each-other through the binary, red first at 42a0210a: on a version-1 project, a later accepted change that edits the earlier change's src/a.rs without declaring it is not named by the earlier change's archive refusal (only the reopen is named); the later change's refusal names the earlier one. The guard test (a hand edit, no later change) passes. cargo test --test integration through one-lane, the neutral git config as a prefix only (#1092).**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 6ec5a3f5f919, observed 2026-11-20T08:40:17Z.
