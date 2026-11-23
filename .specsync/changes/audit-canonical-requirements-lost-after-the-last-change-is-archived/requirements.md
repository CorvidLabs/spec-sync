---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: requirements
---

# Requirements

An archive-only branch can still lose a canonical requirement during conflict resolution. An operator running audit should see every unexplained loss, its original file and the command that displays the original block.

Use the same requirement identity, source-file selection, approved-removal rules and bounded historical reads as REQ-change-169. Credit a removal authorized by a package archived on this branch after the comparison base. Moving a requirement between canonical files keeps its identity. A pre-existing archive mentioning an ID cannot authorize a later removal.
