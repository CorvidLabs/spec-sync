---
change: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
artifact: testing
---

# Testing

Run `validate-release-version.py` (expects published default 6.1.2 everywhere it checks and no archive-gate `@main` pins), `validate-workflow-runtime-pins.py`, `test-validate-release-candidate.py`, `change check --commit`, `change audit` and `fledge lanes run pre-push`, recording actual exits.

| Existing requirement | Evidence |
|----------------------|----------|
| REQ-github-002 (bytes unchanged) | Published v6.1.2 assets verified; validator agreement on the 6.1.2 default and docs; after merge, omitted-input Linux and macOS consumers at the merged wrapper SHA precede the floating v6 move. |

Native requirement_ids is expected to be empty for this Purpose-only delta. No result is inferred from historical receipts.
