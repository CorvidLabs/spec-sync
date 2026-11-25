---
kind: evidence
id: 2026-11-25-at-a6c79b5a-under-the-native-lane-lock-one-real-slice-through
aliases: []
claim: "At a6c79b5a, under the native lane lock: one real slice through the new lane and slice script with the pinned cargo-nextest 0.9.151 (slice 1 of 4: 1047 tests run, 1047 passed, counts file 1047 of 4187), then fledge lanes run verify, 11 steps in 34m14s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check, and the seven CI contract suites (98 release-candidate tests, 7 workflow time-limit tests). No src, tests, .github, Cargo or fledge.toml path changed between that commit and a29b4a43, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 45) pass. No test was removed, ignored or changed in what it asserts about the product."
result: passed
evidence_kind: product-test
revision: a29b4a439c9f19b5103d30b5c40e4c32c010d678
content: 253180fc8d09f289ae64634152d74aece54980851c7599e69f736ab6f8039bed
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two tests at once
fingerprint: sha256:2a8ef991300e2c9c41cc6d6ba264a2559b1cfe1061d263d6c1ae60e012375d1e
external: ext-13684c
criteria: [SPEED-2.b.2]
requirements: []
contracts: []
limitations: []
observed: 2026-11-25T04:33:23Z
actor:
  - agent: claude-code
    terminal: t14v
---
**At a6c79b5a, under the native lane lock: one real slice through the new lane and slice script with the pinned cargo-nextest 0.9.151 (slice 1 of 4: 1047 tests run, 1047 passed, counts file 1047 of 4187), then fledge lanes run verify, 11 steps in 34m14s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check, and the seven CI contract suites (98 release-candidate tests, 7 workflow time-limit tests). No src, tests, .github, Cargo or fledge.toml path changed between that commit and a29b4a43, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 45) pass. No test was removed, ignored or changed in what it asserts about the product.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two tests at once at revision a29b4a439c9f, observed 2026-11-25T04:33:23Z.
