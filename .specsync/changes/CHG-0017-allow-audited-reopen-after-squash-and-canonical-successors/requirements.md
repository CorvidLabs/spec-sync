---
change: CHG-0017-allow-audited-reopen-after-squash-and-canonical-successors
artifact: requirements
---

# Requirements

## REQ-change-018

Audited reopening SHALL recognize canonical acceptance recorded in current Git history after squash integration or complete later canonical governance.

Acceptance Criteria

- Definition digest, passed evidence, closing approval, stale delivery inputs, actor, and reason remain mandatory.
- An unreachable verification commit is allowed only when current history records acceptance or later recorded canonical changes govern every affected spec and path.
- Arbitrary off-history evidence remains rejected.
