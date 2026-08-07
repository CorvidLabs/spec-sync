---
change: CHG-0096-floor-change-sequences-from-remote-ledger-and-document-multi-clone-base
artifact: requirements
---

# Requirements

Change sequence allocation must floor on the remote default-branch sequence ledger when the clone has fetched it, and must honor SPECSYNC_SEQUENCE_BASE for concurrent multi-clone fleets that cannot see each other.
