---
kind: evidence
id: 2026-11-07-focused-fixed-code-run-fledge-run-test-late-close-nocapture
aliases: [OBS-product-test-943d52f230af9865e7c1fe58]
claim: "Focused fixed-code run: fledge run test -- late_close_ -- --nocapture, pinned Rust 1.89.0, CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 nice -n 10; 4 unit and 11 integration tests passed, including all five new destination-symlink regressions, refusal recovery, unchanged outside/source bytes and existing rollback controls."
result: passed
evidence_kind: product-test
revision: 9532a256b4b2e6c39fee88e39ebb212bb3125e18
content: 0077e9b4ea007f052780fd94a72ffb328f842dac0bcaa7ea970301d71212dc9a
environment: local-macos-arm64
fingerprint: sha256:342ca151390ea4435a30150af0e373b2bb9e5d0a7aae229af6d7f4055e8bc4bc
criteria: [SHIP-12]
requirements: [REQ-change-143]
contracts: [specs/change/change.spec.md]
limitations:
  - Focused checks only; full verify and hosted CI pending. Unix symlink tests do not qualify Windows or arbitrary concurrent external filesystem replacement.
observed: 2026-11-07T06:51:19Z
---
**Focused fixed-code run: fledge run test -- late_close_ -- --nocapture, pinned Rust 1.89.0, CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=4 nice -n 10; 4 unit and 11 integration tests passed, including all five new destination-symlink regressions, refusal recovery, unchanged outside/source bytes and existing rollback controls.**

Recorded as passed in local-macos-arm64 at revision 9532a256b4b2, observed 2026-11-07T06:51:19Z.
