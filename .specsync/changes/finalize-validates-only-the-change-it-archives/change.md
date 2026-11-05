---
id: finalize-validates-only-the-change-it-archives
state: implementing
type: bug_fix
base_commit: 0723bdca6c7d592baff6fdd8dba84b2de07c875a
---

# Finalize validates only the change it archives

## Intent

Finalize validates only the change it archives

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- One finalize spawns no more Git processes when more archived changes exist; every refusal finalize issues for the change it archives is unchanged; change audit --strict still names a hidden workflow-version downgrade in another archive; a Raven-sized finalize finishes in seconds.

## No-spec Rationale

Not applicable
