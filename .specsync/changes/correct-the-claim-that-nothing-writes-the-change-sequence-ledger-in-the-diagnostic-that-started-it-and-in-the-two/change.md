---
id: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-two
state: implementing
type: bug_fix
base_commit: 4b72b09de0e950b7a0479463dbefcac33d516cac
---

# Correct the claim that nothing writes the change sequence ledger, in the diagnostic that started it and in the two places it was copied to

## Intent

Correct the claim that nothing writes the change sequence ledger, in the diagnostic that started it and in the two places it was copied to

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- No live file in the repository claims that nothing writes .specsync/change-sequence.json or that it is read-only or frozen; the diagnostic at src/change.rs states the true and useful constraint (nothing allocates, so it cannot be repaired by minting a higher sequence); and AGENTS.md and CHANGELOG.md name floor_sequence_ledger_to_committed as the one writer and the single direction it may move the ledger.

## No-spec Rationale

The change module owns src/change.rs, so ownership is declared explicitly with --spec; but the edit is a diagnostic string and two prose copies of it, so no canonical spec text moves and no requirement gains, loses or alters a guarantee.
