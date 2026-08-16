---
change: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
artifact: tasks
---

# Tasks

1. Split the un-archive move out so the source path is known to the caller.
2. Run the reopen body in a helper; restore the package on any error.
3. Suffix the refusal so the restore is visible.
4. Invert the transactional half of sandbox drill 008, which pins the old
   behaviour; leave its anchor-preflight half pinned, because that still
   reproduces.
