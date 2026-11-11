---
change: delta-materializer-matches-titled-requirement-headers
artifact: testing
---

# Testing

Unit tests in `src/change_tests/`: a MODIFIED delta applies to a titled
requirement header; the title is normalized away on write; a `## ` section
delta does not match a section whose heading merely starts with the key;
`delta_item_is_applied` recognizes the already-applied titled block.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-150 | change_tests unit tests exercising MODIFIED against titled and untitled headers |
