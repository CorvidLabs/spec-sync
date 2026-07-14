---
change: CHG-0029-address-all-remaining-review-feedback-from-pr-366
artifact: testing
---

# Testing

## Requirement Evidence

- `REQ-change-030`: focused unit tests cover scope prompting, registry coverage/evidence, protected registry edits, disabled policy, and Cargo command discrimination.
- `REQ-cli-004`: a CLI regression proves inherited nested `check` fails before handler-specific work.

## Planned Verification

- Reproduce each unresolved PR #366 thread with a failing regression before applying its fix.
- Run focused `change` and binary-dispatch test groups.
- Run formatting, Clippy, all unit/integration tests, release build, strict SpecSync, Trust, Augur, and Attest.
