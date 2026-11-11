---
change: re-approval-accepts-an-unchanged-delta-body-that-already-carries-the-moved-canonical-text
artifact: tasks
---

# Tasks

- [x] Recover the approval-time base block from git by its recorded digest
- [x] Three-way comparison (LCS-aligned on the base) accepting only bodies that carry every moved edit
- [x] Accept and record the new base in `approval_delta_bases`; keep the refusal otherwise
- [x] Unit tests for the comparison; git-backed approval tests for the #975 case and its guard
