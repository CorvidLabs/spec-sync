---
change: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
artifact: design
---

# Design

Before an already-applied verifying change prepares an empty canonical write set, acceptance loads the latest reopening event and compares the current definition digest with the prior accepted verification contract digest. A mismatch fails before a new closing approval or any file write.

Missing reopen history also fails closed because `canonical_applied` in a verifying record is meaningful only after an audited reopen. Initial acceptance remains unchanged because its marker is false until canonical application completes. Repeated delivery-only reopen cycles compare against the latest prior verification and continue to preserve append-only evidence.
