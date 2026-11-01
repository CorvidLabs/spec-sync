---
change: acknowledge-anchored-historical-receipts-explicitly-without-weakening-normal-transition-proof
artifact: testing
---

# Testing

## Acceptance Criteria

Use a genuine legacy receipt whose files change before creating its successor: normal succession must still refuse, while only a newly approved historical request can close. Verify exact old package preservation, explicit schema/status, and fresh verification/review requirements. Negative controls cover missing or non-ancestor anchors, non-accepted or tampered packages/receipts, forged witness digests, false proven claims, stale successor bytes, protected paths and unsupported entry types. Run pinned Rust 1.89 full Fledge verification, strict specs/full configured coverage, pre-push, Augur, Attest and Trust; qualify the real consumer separately.


## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-118 | historical_reconciliation_* lifecycle and negative controls in src/change_tests/historical_reconciliation.rs |
| REQ-cmd-change-025 | Actual consumer CLI status/finalize JSON inspection and unchanged normal output controls |
| REQ-cli-args-020 | change_historical_reconciliation_requires_explicit_actor_and_reason |
