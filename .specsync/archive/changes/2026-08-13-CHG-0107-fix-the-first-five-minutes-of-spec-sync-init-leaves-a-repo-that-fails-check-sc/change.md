---
id: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
state: archived
type: bug_fix
base_commit: 4ddf810ed39b56482ba96a0d209ee9216ef56fe9
---

# Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green

## Intent

Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green

## Affected Canonical Specs

- `change`
- `cmd_init`
- `cmd_issues`
- `generator`
- `validator`

## Acceptance Criteria

- A repository initialized with `specsync init` passes `specsync check --strict` with exit 0 before any spec is authored. A module created with `specsync scaffold` passes `specsync check --strict` with exit 0 while its sections still hold generated placeholder prose, and a section a change actually authored then emptied still fails. A directory listed in a spec's `files:` block resolves to the source files beneath it for validation, coverage, and export extraction instead of being silently skipped, and an unresolvable directory reports an error rather than passing green.

## No-spec Rationale

Not applicable
