---
id: let-change-check-commit-a-project-rooted-in-a-repository-subdirectory
state: archived
type: bug_fix
base_commit: 884a33d3c0cb95680dae786ff25688569ee16c96
---

# Let change check commit a project rooted in a repository subdirectory

## Intent

Let change check commit a project rooted in a repository subdirectory

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- In a project rooted in a subdirectory of its Git repository, `change check <id> --commit` on a change whose source and materialized spec live in the project completes: it commits the materialized spec and the verification evidence, as it does at the repository root.
- Working-tree evidence capture reads Git's modified-path output relative to the project, so the change's own spec (`app/specs/<module>/<module>.spec.md` from Git, `specs/<module>/<module>.spec.md` in the project) is in scope.
- A path Git reports outside the project or outside the change's candidates is still refused with the same out-of-scope error naming that path.
- A project at the repository root behaves exactly as before. No gate is loosened.
- The auditor's subdirectory steps are an integration test that fails on main first.

## No-spec Rationale

Not applicable
