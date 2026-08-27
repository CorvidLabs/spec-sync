---
id: retire-the-spec-text-describing-the-deleted-change-sequence-allocation-including-specsync-sequence-base
state: archived
type: documentation
base_commit: d6f266a4fd683246469eb15a8f632061dd5cfbb4
---

# Retire the spec text describing the deleted change-sequence allocation, including SPECSYNC_SEQUENCE_BASE

## Intent

retire the spec text describing the deleted change-sequence allocation, including SPECSYNC_SEQUENCE_BASE

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- REQ-change-055 is absent from specs/change/requirements.md: the allocation floor, the remote high-water floor and SPECSYNC_SEQUENCE_BASE it describes were all deleted by the ordinal retirement (#665) and none of them is stated anywhere else that survives.
- REQ-change-071 is absent: its normative SHALL is directly reversed by REQ-change-072 and its implementation criterion names the deleted allocation floor as its source.
- No canonical change spec text asserts that a sequence is allocated, that a next ID is claimed, or that a newly created change generates a ledger claim; the sibling clauses in REQ-change-022, REQ-change-026, REQ-change-070 and REQ-change-072, in the change.spec.md invariants and in context.md all describe the read-only ledger that actually ships.
- AGENTS.md no longer instructs agents to set SPECSYNC_SEQUENCE_BASE or to expect change new to floor on a remote ledger, and instead states the slug identity model and the read-only ledger rule.
- The ledger invariants that do still hold and are still enforced are left intact and attributed: the commit-side floor in REQ-change-070 and the branch-own-history gate in REQ-change-072.

## No-spec Rationale

Not applicable
