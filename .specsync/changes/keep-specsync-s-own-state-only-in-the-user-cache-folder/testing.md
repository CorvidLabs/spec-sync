---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: testing
---

# Testing

Fixtures are temporary directories. "Outside" is a sibling directory there. Integration tests set `XDG_CACHE_HOME` to a folder in the system temporary directory, outside every fixture project.

## Requirement evidence

Filled in as the tests are written.
