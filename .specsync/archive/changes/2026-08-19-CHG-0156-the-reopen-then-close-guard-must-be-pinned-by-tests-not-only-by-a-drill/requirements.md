---
change: CHG-0156-the-reopen-then-close-guard-must-be-pinned-by-tests-not-only-by-a-drill
artifact: requirements
---

# Requirements

## REQ-change-078 (new)

The rule governing where committed scoped review evidence may move SHALL be pinned by tests that
fail when either the permitted directions or the refusal itself is removed.

See `deltas/change.md` for the canonical delta.

## Deliberately unchanged

No product behaviour. `REQ-change-073` already states the rule; this change makes its removal
detectable.
