---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: tasks
---

# Tasks

- [ ] Reproduce the missing-requirement case through public audit and strict check on a full-history branch with zero active workspaces. Keep a separate no-loss fixture on the same base.
- [ ] Expose one branch comparison from the #1108 reader to audit and strict check. Preserve per-change checks, plain-check drift, and the existing required Lifecycle job.
- [ ] Cover authorized branch removals, an older archive, a moved requirement, a managed project whose only CI gate is strict check, an SDD-opt-out control, and missing or incomplete comparison history.
- [ ] Update both module contracts and testing companions, then run the scoped and project verification lanes.
