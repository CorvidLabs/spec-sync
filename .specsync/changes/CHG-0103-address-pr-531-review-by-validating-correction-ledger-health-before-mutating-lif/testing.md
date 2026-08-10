---
change: CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif
artifact: testing
---

# Testing

Focused Rust tests will corrupt an existing change's correction ledger, attempt each affected
mutation, and assert both a safe error and byte-for-byte unchanged lifecycle files. Existing
read-only fail-closed coverage remains required.

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-010 | `cargo test --test integration change::invalid_correction_ledger_blocks_mutating_commands_before_persistence -- --exact --nocapture` exercises all three real CLI mutations and passes; `cargo test commands::change::` retains the read-only safe-diagnostic coverage |

Scoped lifecycle verification runs `cargo test commands::change::`, followed by strict
lifecycle/spec validation; the exact integration regression is also run explicitly before that gate.
