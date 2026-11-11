---
change: journal-read-errors-name-the-file-and-a-quoted-recovery
artifact: testing
---

# Testing

change_tests/transaction_journal_error.rs gains non-UTF-8, directory and
spaced-root cases.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-151 | change_tests journal tests incl. spaced-root recovery |
