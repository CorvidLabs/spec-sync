---
id: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
state: approved
type: feature
base_commit: 99f61b2b5667d60c3b0705902486f910c8e067c5
---

# Rebase a stale MODIFIED requirement delta automatically when main's edit does not overlap it, keeping the approval

## Intent

Rebase a stale MODIFIED requirement delta automatically when main's edit does not overlap it, keeping the approval

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- When main edits a requirement a MODIFIED delta was approved against and the two edits do not overlap word by word, change check rewrites the delta to the merge, records a verifiable rebase in approvals.json, keeps the approval and materializes the merged text; when they overlap it refuses as today and asks for re-approval; change status shows each rebase

## No-spec Rationale

Not applicable
