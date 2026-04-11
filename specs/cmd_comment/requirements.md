---
spec: cmd_comment.spec.md
---

## User Stories

- As a developer, I want `specsync comment` to generate a clear spec-check summary so I can see validation status at a glance in my PR
- As a CI operator, I want clear exit codes and error messages so that pipeline failures are actionable
- As a marketplace action user, I want the same comment output as the project's own CI so there are no discrepancies between invocation methods

## Acceptance Criteria

- `cmd_comment` runs full validation and renders the check summary as GitHub-flavored markdown
- When `--pr` is omitted, markdown is printed to stdout for piping (used by both the marketplace action and CI workflow)
- When `--pr N` is set, the comment is posted directly to the specified PR via `gh pr comment`
- Exits 1 if `gh` CLI fails or the GitHub repo cannot be resolved
- The marketplace action (`action.yml`, `comment: true`) and CI workflow (`.github/workflows/ci.yml`) both invoke `specsync comment` in stdout mode — no alternative comment generation paths exist

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing
- Single source of truth: all PR comment content must flow through `specsync comment` to guarantee identical output across integrations

## Out of Scope

- GUI or web interface
- Interactive prompts
- Posting comments through any path other than `specsync comment` + `gh`
