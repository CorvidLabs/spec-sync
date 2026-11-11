---
change: audit-reports-requirements-traversal-errors
artifact: testing
---

# Testing

Unit test: an unreadable specs subdirectory produces a traversal error naming
the path.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-validator-046 | change_tests unit test with a chmod-0 subdirectory |
