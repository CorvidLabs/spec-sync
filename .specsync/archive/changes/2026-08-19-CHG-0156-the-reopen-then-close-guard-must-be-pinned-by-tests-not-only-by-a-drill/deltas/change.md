## ADDED

### REQUIREMENT REQ-change-078

The rule governing where committed scoped review evidence may move SHALL be pinned by tests that fail when either the permitted directions or the refusal itself is removed.

Acceptance Criteria
- Removing the archive-to-active direction fails a test, so the defect where a reopened change could never be closed again cannot return silently.
- Deleting the guard entirely fails a test, and fails a different one than the direction removal does, so a fix and the refusal it lives inside are pinned independently.
- A move to any location other than a change's active workspace and its archive is refused, asserted in both directions.
- Deleting committed review evidence is refused.
