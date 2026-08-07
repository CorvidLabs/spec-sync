---
id: CHG-0096-floor-change-sequences-from-remote-ledger-and-document-multi-clone-base
state: approved
type: feature
base_commit: a679cf733296759cea216aaf72f355570bb14ef0
---

# Floor change sequences from remote ledger and document multi-clone BASE

## Intent

floor change sequences from remote ledger and document multi-clone BASE

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- maximum_observed_sequence floors on remote default-branch change-sequence ledger when readable; SPECSYNC_SEQUENCE_BASE still raises high-water; Agents.md documents multi-clone; unit test for remote floor

## No-spec Rationale

Sandbox #10: multi-clone change new collides on sequence. Floor from remote high-water when available; document SPECSYNC_SEQUENCE_BASE for concurrent fleets.
