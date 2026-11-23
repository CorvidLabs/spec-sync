---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: context
---

# Context

An archive-first CI branch with 442 archived packages and no active workspaces lost REQ-cmd-change-040. Both strict audit modes returned empty errors and warnings. See the #1105 reproducer and its separate no-loss control. This draft is stacked on #1108 at 5314fb6a for its bounded historical requirement reader.

The maintainer answered the #1105 interview with “Both `change audit` and `check --strict`, so projects running either command catch the loss”. HI records the answer as CHANGE-17.d under the existing requirement-preservation intent.

The required Lifecycle gate already runs `change audit --strict` on full history after Archive. Projects may instead run only `check --strict`. Both commands should call the same branch comparison and report the same lost IDs and restore guidance.
