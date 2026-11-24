---
kind: evidence
id: 2026-11-20-428-ship-order-at-5ef1af6a
aliases: []
claim: "Codex round 2 P1 and corvid-agent's blocker on #1120: with verification stale too, ship-status puts a ledger refusal whose way out is not check --commit first and makes that way out ship_next, so ship names the restore and never check --commit. Red first through the binary at 8e3eb2ee; a refusal that check --commit itself resolves keeps its place (review_and_ship_refuse_a_verification_commit_that_lacks_the_work, which the first fix broke at 9f2a8f18). Checked at 5ef1af6a: fledge lanes run verify 10/10 in 7m19s through one-lane (unit 3127, 2 ignored; integration 646; spec check 62/62, 168/168 files), the neutral git config as a prefix only (#1092); change audit --strict passes."
result: passed
evidence_kind: product-test
revision: 5ef1af6aed26f27a373c159c1d676f65383882eb
content: 7ee8f21a4dd3074f9804fb9770670f3d30d213e8bb4cdf8881cc03620b406965
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:2db7084bc9d5c6a0dac38539446cd9dcb5aff0f9f8b05a6eb6cdeca0a573f0db
external: ext-36d368
criteria: [CHANGE-16]
requirements: []
contracts: [specs/change/change.spec.md, specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-20T08:12:32Z
actor:
  - agent: claude-code
    terminal: t59
---
**Codex round 2 P1 and corvid-agent's blocker on #1120: with verification stale too, ship-status puts a ledger refusal whose way out is not check --commit first and makes that way out ship_next, so ship names the restore and never check --commit. Red first through the binary at 8e3eb2ee; a refusal that check --commit itself resolves keeps its place (review_and_ship_refuse_a_verification_commit_that_lacks_the_work, which the first fix broke at 9f2a8f18). Checked at 5ef1af6a: fledge lanes run verify 10/10 in 7m19s through one-lane (unit 3127, 2 ignored; integration 646; spec check 62/62, 168/168 files), the neutral git config as a prefix only (#1092); change audit --strict passes.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 5ef1af6aed26, observed 2026-11-20T08:12:32Z.
