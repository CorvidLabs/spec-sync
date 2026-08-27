---
id: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-three
state: implementing
type: documentation
base_commit: dccd82105956d62df76bb4fec9fb777c4b31f15b
---

# Correct the claim that nothing writes the change sequence ledger, in the diagnostic that started it and in the three places it was copied to

## Intent

Correct the claim that nothing writes the change sequence ledger, in the diagnostic that started it and in the three places it was copied to

## Affected Canonical Specs

- None

## Acceptance Criteria

- No live file in the repository claims that nothing writes .specsync/change-sequence.json or that it is read-only or frozen; the diagnostic at src/change.rs:2189 states the true and useful constraint (nothing allocates, so it cannot be repaired by minting a higher sequence); and AGENTS.md and CHANGELOG.md name floor_sequence_ledger_to_committed as the one writer and the direction it may move the ledger.

## No-spec Rationale

Corrects a false factual claim in a diagnostic string and two prose copies of it; no canonical spec text changes and no requirement gains, loses or alters a guarantee.
