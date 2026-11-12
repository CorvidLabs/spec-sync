---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: requirements
---

# Requirements

The delta adds REQ-change-158 to `specs/change/requirements.md` (`deltas/change.md`). The coordinator reserved that id and change-spec invariant 60 in the allocation file.

It changes no existing requirement. REQ-change-079 still reports an unknown workflow version as written by a newer SpecSync; version 3 is now known, and a version above 3 is still unknown. REQ-change-relayed-decision-001 is unchanged: the relay is still a claim no gate reads, and only the record's workflow version moves. The workflow-v2 adoption requirement already refuses a workflow-version downgrade, and 3 back to 2 is one.
