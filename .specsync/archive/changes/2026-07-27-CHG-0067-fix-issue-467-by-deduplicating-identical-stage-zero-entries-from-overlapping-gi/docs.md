---
change: CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi
artifact: docs
---

# Docs

## Canonical Documentation

Add `REQ-change-042` through the `change` semantic delta and update the change module's context,
tasks, and testing companions at acceptance. The contract states that identical stage-zero records
from overlapping bounded pathspec batches are idempotent while differing mode or object pairs fail
closed.

## User-Facing Documentation

No CLI syntax or standalone documentation page changes. The observable fix is that valid lifecycle
verification and acceptance scopes no longer fail with `duplicate Git index stage-zero entry`
solely because a parent directory and exact children overlap across batches.

## Issue and Delivery Notes

Reference GitHub issue #467 and report the focused cross-batch, conflicting-mode, and
conflicting-object test results with the lifecycle and trust evidence.
