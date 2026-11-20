---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: testing
---

# Testing

Fixtures are temporary directories. Every link points inside the fixture's own temporary directory, and "outside" is a sibling directory there. Integration tests set `XDG_CACHE_HOME` to a folder in the fixture, so nothing is written to the user's cache.

## Requirement evidence

Filled in as the tests are written.
