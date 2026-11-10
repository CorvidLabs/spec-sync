---
id: scaffold-prints-the-empty-files-warning-when-every-candidate-source-is-owned-by-another-spec
state: implementing
type: bug_fix
base_commit: 1fffafcbcf05e179b0d92139d709a6bf146abcb8
---

# Scaffold prints the empty-files warning when every candidate source is owned by another spec

## Intent

scaffold prints the empty-files warning when every candidate source is owned by another spec

## Affected Canonical Specs

- `cmd_scaffold`

## Acceptance Criteria

- scaffold foo, where every candidate source file is already listed by another spec, writes files: [] and prints the same empty-files warning on stderr that new prints; the scaffold, single-source-fallback and new ownership integration tests assert files: [] and that warning

## No-spec Rationale

REQ-cmd-scaffold-006 AC2 already requires files: [] plus the empty-files warning when no unowned candidate is left; scaffold omitted the warning, and this change makes the code match the existing requirement
