---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: testing
---

# Testing

Focused Rust tests will corrupt an existing change's correction ledger, attempt each affected
mutation, and assert both a safe error and byte-for-byte unchanged lifecycle files. Existing
read-only fail-closed coverage remains required.

A deterministic domain test will hold the project lock, start an answer mutation, corrupt the
ledger while that mutation waits, release the lock, and require the post-lock validation to reject
the mutation without changing state or rendered change bytes.

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-010 | `cargo test --test integration change::invalid_correction_ledger_blocks_mutating_commands_before_persistence -- --exact --nocapture` passes against all three real CLI mutations; `fledge run test -- commands::change::` passes 4 command-layer tests and retains read-only safe-diagnostic coverage |
| REQ-change-057 | `fledge run test -- change::tests::mutation_rechecks_correction_ledger_after_lock_acquisition` passes the deterministic corruption-while-waiting regression and proves post-lock rejection without persistence |

Scoped lifecycle verification runs `cargo test commands::change::`, followed by strict
lifecycle/spec validation; the exact integration regression is also run explicitly before that gate.
