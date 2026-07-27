---
change: CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi
artifact: requirements
---

# Requirements

## Problem

Issue #467 shows that valid lifecycle delivery scopes can fail solely because overlapping parent
and child pathspecs land in different bounded Git query batches. The repeated records describe the
same index entry and are not unresolved index stages.

## Required Outcomes

- Accept and deduplicate repeated stage-zero observations only when both mode and normalized object
  ID match the first observation.
- Reject a repeated path when either its mode or object ID differs.
- Preserve the first observed pair when a conflicting duplicate is rejected.
- Keep the existing deterministic bounds and all unrelated Git evidence validation unchanged.
- Allow lifecycle verification and acceptance to inspect valid overlapping delivery scopes without
  the prior duplicate-stage-zero failure.

## Compatibility

- No public type, function, command, state, or persisted evidence schema changes.
- Existing unresolved-stage and malformed-index failures remain fail closed.
- The prior unconditional duplicate error narrows to conflicting duplicate pairs only.
