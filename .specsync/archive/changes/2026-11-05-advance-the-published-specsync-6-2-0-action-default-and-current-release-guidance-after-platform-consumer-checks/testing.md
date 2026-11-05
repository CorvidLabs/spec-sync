---
change: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
artifact: testing
---

# Testing

Run `validate-release-version.py` (expects published default 6.2.0 everywhere it checks), `validate-workflow-runtime-pins.py`, `test-validate-release-candidate.py`, `change check --commit`, `change audit` and `fledge lanes run pre-push`, recording actual exits.

| Existing requirement | Evidence |
|----------------------|----------|
| REQ-github-002 (bytes unchanged) | Published v6.2.0 assets verified; validator agreement on the 6.2.0 default and docs; after merge, omitted-input Linux and macOS consumers at the merged wrapper SHA precede the floating v6 move. |

Native requirement_ids is expected to be empty for this Purpose-only delta. No result is inferred from historical receipts.
