---
change: the-verifying-handoff-must-not-claim-the-verification-evidence-is-committed
artifact: tasks
---

# Tasks

- [x] Reword the Verifying-safe reason in `classify_handoff` (`src/change.rs`) so it names the
      implementation as committed and the verification as current
- [x] Pin it in `handoff_verifying_follows_evidence_currency` (`src/change_tests.rs`): no
      Verifying reason may claim the verification is committed
- [x] `cargo test --release --bin specsync handoff`, `cargo clippy -- -D warnings`, `cargo fmt --check`
