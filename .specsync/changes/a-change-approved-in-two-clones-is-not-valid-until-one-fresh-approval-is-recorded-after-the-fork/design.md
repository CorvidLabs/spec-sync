---
change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
artifact: design
---

# Design

- **Where:** inside definition-approval validation (`ensure_definition_approval_valid`), which `change check`, `change review`, acceptance in `change finalize` and `change ship`, `change status` (approval currency) and `change audit` already call.
- **The walk:** every committed version of the change's `approvals.json` that `HEAD` reaches, with `git log --full-history`, so both sides of every merge are read. Each version's approvals are compared without `relayed_by`.
- **A fork:** two versions, neither extending the other, where at some position each holds an approval the other lacks. It counts only while the definition approval that counts now appears in either forked version. One definition approval recorded after both, appearing in neither, settles it.
- **The refusal:** names both approvers, the two commits, and `specsync change approve <id> --actor <name>`.
- **Out of the walk:** archived changes are not judged again, and only definition approvals are compared. A merge resolution that reorders records stays #428's rewrite, refused at the merge commit.
