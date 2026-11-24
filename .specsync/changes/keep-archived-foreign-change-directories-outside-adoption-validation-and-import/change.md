---
id: keep-archived-foreign-change-directories-outside-adoption-validation-and-import
state: approved
type: bug_fix
base_commit: 892df7f5e00e941edf6d00d9d8368080347b0281
---

# Keep archived foreign change directories outside adoption validation and import

## Intent

Keep archived foreign change directories outside adoption validation and import

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- For both OpenSpec and Spec Kit, preview and import ignore a symlink inside the real top-level archive directory and reject a symlink in an active change. Preview and refusal leave the destination unchanged. Real adoption imports only active content.

## No-spec Rationale

Not applicable
