---
change: cover-the-torn-journal-assertion-update
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-151 | the covered assertion passes in the full suite |

## How the covered assertion runs

`change check` executes only the in-process spec↔code sync (recorded as
`specsync check --spec change`); sdd.json verification commands are not
executed by the lifecycle. The assertion this change owns runs as
`cargo test torn_canonical_transaction_journal_fails_closed` — locally at the
final head and in hosted CI's `test` job.
