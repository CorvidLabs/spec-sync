# Lesson bundle: every-actor-accepting-verb-normalizes-identity-the-same-way

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Every actor-accepting verb normalizes identity the same way
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, specs/change/, tests/
- **Acceptance**: send-back, reopen, correct and correct-owner reject actor names containing newlines or control characters and trim padding, exactly as approve/review do through resolve_actor

## Evidence

- Verification commit: `b001f287df14b99ca47b19e712044ca4ab6784ba`
- Base commit: `9ef1a373f316ae8b320f4ffc0b18bc31ed0cf9f2`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context






Round-2 review of #426 (PR #1026): resolve_actor covers approve/review/accept, but send-back
(src/change/send_back.rs:278), reopen (sdd_version.rs:4057), correct
(historical_definition_digest_matches.rs:940) and correct-owner (sdd_version.rs:4610) only
trimmed and checked empty — a newline actor still entered the append-only ledger.

## From the change's testing.md

# Testing


Integration: `change send-back --actor "Ada\nInjected"` fails and approvals.json is unchanged;
reopen/correct-owner reject the same. Padded actor is stored trimmed.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | integration `change_approve_trims_and_validates_actor_identity` (extended: send-back with a newline actor fails and the ledger is byte-unchanged); unit send_back suite 10/10 |

## Where these lessons go

- `specs/change/change.3md`
