---
id: keep-3md-files-from-being-classified-as-roff
state: archived
type: bug_fix
base_commit: f60b5ea20f360c8b6db5b536abbc7be88ecffd70
---

# Keep 3md files from being classified as Roff

## Intent

Keep 3md files from being classified as Roff

## Affected Canonical Specs

- `cmd_init`

## Acceptance Criteria

- A new spec-sync project gets *.3md linguist-language=Text, and a project that already classifies *.3md is left alone.

## No-spec Rationale

Not applicable
