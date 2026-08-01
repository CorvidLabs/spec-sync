---
change: CHG-0072-heal-reopen-closing-approval-recovery-for-stale-accepted-evidence
artifact: testing
---

# Testing

## Automated evidence

- `cargo test change::tests::` — reopen and finalize recovery tests green
- Requirement coverage:
  - REQ-change-034 — reopen binds historical verification; re-accept restores closing
