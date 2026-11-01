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


Executed qualification: six targeted checks and the complete Fledge verify lane passed (2,697 unit, 478 integration, 66 release-contract and 35 CI-contract tests) with pinned Rust 1.89. The isolated missing-digest-binding mutation failed the old-approval assertion. Rust/build-input equality was checked after the separately landed bounded recovery merged into main. Documentation and actual consumer archival results are recorded separately as they complete. No human code review or published release is claimed.
