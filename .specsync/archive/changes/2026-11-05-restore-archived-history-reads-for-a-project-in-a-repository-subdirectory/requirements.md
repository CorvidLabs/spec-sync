---
change: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory
artifact: requirements
---

# Requirements

The existing requirements already say what the history read must do. This change restores them when the project root is not the Git repository root. No new requirement id.

REQ-change-043. Workflow-origin history follows every reachable canonical dated archive state path for the change id. The acceptance criteria now also say that this read uses repository-root pathspecs, so a subdirectory project still sees every dated path and still refuses a workflow-version downgrade that a later commit restored.

REQ-change-119. A cached workflow-version verdict is reused only when no commit since it touched that change's state paths. The acceptance criteria now also say that the invalidation walk uses `:(top,literal)`, so a subdirectory project does not reuse a cached pass across that kind of commit.
