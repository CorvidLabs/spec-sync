---
kind: evidence
id: 2026-11-12-req-change-158-approved-materialized-by-this-change-s-check-d5caa2
aliases: [OBS-product-test-ce78f3720e9d6d6ae3c9b0a4]
claim: "REQ-change-158 (approved, materialized by this change's check): macOS 26, same tree: cargo test --test integration relayed_decision passed 29, 6 of them new. cargo test --bin specsync relay passed 25, 3 of them new. Targeted --bin filters all passed: workflow 29, archive 122 (1 ignored), history 34, unknown 25, send_back 10, retire 22, reopen 34, correct 31. Integration change:: 75, closed_late 13, lifecycle 5. cargo clippy --all-targets --locked -D warnings is clean; cargo fmt --check passes; specsync check --strict --require-coverage 100 --force passes (62 specs, 100%). The CI script tests pass (required gate, trigger paths, clippy targets, Linux baseline, runtime pins, release version), and so does actionlint on ci.yml."
result: passed
evidence_kind: product-test
method: automated
revision: 09434150020a93ff999b146e98733f60f8b94b8a
content: a7a56489d7c34e4e0d0ac57dcd62e98c88443d9ed71e5299ed68b07cfd042120
environment: local
criteria: [CHANGE-13]
requirements: [REQ-change-relayed-decision-001]
contracts: [specs/change/change.spec.md]
limitations:
  - "The full cargo test suite was not run locally (coordinator rule: full suites run in hosted CI)"
observed: 2026-11-12T02:52:57Z
---
**REQ-change-158 (approved, materialized by this change's check): macOS 26, same tree: cargo test --test integration relayed_decision passed 29, 6 of them new. cargo test --bin specsync relay passed 25, 3 of them new. Targeted --bin filters all passed: workflow 29, archive 122 (1 ignored), history 34, unknown 25, send_back 10, retire 22, reopen 34, correct 31. Integration change:: 75, closed_late 13, lifecycle 5. cargo clippy --all-targets --locked -D warnings is clean; cargo fmt --check passes; specsync check --strict --require-coverage 100 --force passes (62 specs, 100%). The CI script tests pass (required gate, trigger paths, clippy targets, Linux baseline, runtime pins, release version), and so does actionlint on ci.yml.**

Recorded as passed in local at revision 09434150020a, observed 2026-11-12T02:52:57Z.
