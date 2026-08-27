---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: tasks
---

# Tasks

- [x] Reproduce the ledger downgrade against the unfixed binary and record what it produced.
- [x] Establish whether the downgrade reaches a canonical spec on workflow v1, and on v2.
- [x] Scan every `approvals.json` under `.specsync/` for the shape the new refusal rejects.
- [x] Record `approved_delta_digests` on both members of the portable 5.0.1 definition pair.
- [x] Refuse a definition approval that records no delta wording when an earlier one recorded it.
- [x] State the monotonicity invariant at the `approved_delta_digests` declaration.
- [x] Add three discriminators and one honestly labelled control.
- [x] Verify every discriminator fails with the fix disabled in place, and the control does not.
- [x] Update `specs/change/*`: contract, invariant, error case, requirement, context, testing.
- [x] `cargo clippy -- -D warnings` bare, `cargo test`, `change check`, `change audit --strict`.
