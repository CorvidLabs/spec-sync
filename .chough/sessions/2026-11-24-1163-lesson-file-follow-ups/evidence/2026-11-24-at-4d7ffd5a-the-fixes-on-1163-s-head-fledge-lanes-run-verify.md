---
kind: evidence
id: 2026-11-24-at-4d7ffd5a-the-fixes-on-1163-s-head-fledge-lanes-run-verify
aliases: []
claim: "At 4d7ffd5a (the fixes on #1163's head), fledge lanes run verify passed 10/10 steps in 30m54s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check and the six CI contract suites. No src, tests or Cargo path changed between that commit and aeabda2a, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 37) pass. On the CLI with the release binary that lane built (sha256 d4bbbec3), against the binary of #1163's head: a plain check after a clean run reports a lesson file broken since (it was skipped before); a directory named <module>.3md is reported as not a regular file (nothing was said before); for main.spec.md with module product, product.3md is the file checked and main.3md is not read (the reverse before); a FIFO or dangling link there still fails closed in discovery; change check passes and change check --strict fails on the warning, on both."
result: passed
evidence_kind: product-test
revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
content: 249d7216b1e401b6abb655cc28397d6d48ef36f70a96b082f233796f1b4c4255
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
fingerprint: sha256:44000d8701e4c3ab7c4f5f054e5abc63e2a888d072a56f6765f4fc68c7eaa9a9
external: ext-47bc73
criteria: [THREE-1]
requirements: []
contracts: []
limitations: []
observed: 2026-11-24T23:27:01Z
actor:
  - agent: claude-code
    terminal: t14v
---
**At 4d7ffd5a (the fixes on #1163's head), fledge lanes run verify passed 10/10 steps in 30m54s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check and the six CI contract suites. No src, tests or Cargo path changed between that commit and aeabda2a, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 37) pass. On the CLI with the release binary that lane built (sha256 d4bbbec3), against the binary of #1163's head: a plain check after a clean run reports a lesson file broken since (it was skipped before); a directory named <module>.3md is reported as not a regular file (nothing was said before); for main.spec.md with module product, product.3md is the file checked and main.3md is not read (the reverse before); a FIFO or dangling link there still fails closed in discovery; change check passes and change check --strict fails on the warning, on both.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads at revision aeabda2a3983, observed 2026-11-24T23:27:01Z.
