---
spec: cmd_comment.spec.md
---

## User Stories

- As a developer, I want `specsync comment` to generate a clear spec-check summary so I can see validation status at a glance in my PR
- As a CI operator, I want clear exit codes and error messages so that pipeline failures are actionable
- As a marketplace action user, I want the same comment output as the project's own CI so there are no discrepancies between invocation methods

## Acceptance Criteria

- `cmd_comment` runs the same validation pipeline as `check` (`run_validation` in collect mode) and renders the summary via `comment::render_check_comment` as GitHub-flavored markdown
- The pass/fail status embedded in the comment is derived from `compute_exit_code` using the same inputs as `check`, so the comment status matches CI exactly
- `--strict`, `--enforcement`, and `--require-coverage` affect the computed status the same way they do for `check`; `--enforcement` overrides config, and `--strict` implies strict enforcement
- When `--pr` is omitted, markdown is printed to stdout for piping (used by both the marketplace action and CI workflow)
- When `--pr N` is set, the repo is resolved via `github::resolve_repo` and the comment is posted via `gh pr comment --repo <repo> --body <body>`
- Exits 1 if `gh` CLI is missing, `gh pr comment` exits non-zero, or the GitHub repo cannot be resolved
- The marketplace action (`action.yml`, `comment: true`) and CI workflow (`.github/workflows/ci.yml`) both invoke `specsync comment` in stdout mode — no alternative comment generation paths exist
- Configured SDD verification commands still execute and fail closed in comment mode, but their child stdout and stderr do not contaminate the markdown body
- The project CI invokes `cargo run --quiet -- comment` and applies a defensive UTF-8-safe byte cap before forwarding the body to action inputs and job outputs

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing
- Single source of truth: all PR comment content must flow through `specsync comment` to guarantee identical output across integrations

## Out of Scope

- GUI or web interface
- Interactive prompts
- Posting comments through any path other than `specsync comment` + `gh`

### REQ-cmd-comment-001

Generated pull-request comments SHALL include SDD lifecycle failures in their status and remediation details.

Acceptance Criteria
- SDD errors and warnings appear in the rendered comment alongside canonical spec validation.
- An SDD-only failure produces a failing comment status.

### REQ-cmd-comment-002

Generated pull-request comments SHALL report SDD lifecycle failures even when a project has no canonical spec files.

Acceptance Criteria
- Empty canonical discovery does not bypass SDD checking in comment mode.
- SDD-only errors render a failing comment with actionable detail.

### REQ-cmd-comment-003

The comment command SHALL emit a bounded markdown protocol on stdout.

Acceptance Criteria

- Configured SDD verification command output is captured away from stdout in comment mode.
- Only the final markdown report is printed when `--pr` is omitted.
- Oversized detail is truncated with a clear remediation message before GitHub's comment limit.
- Malformed Gradle/manifest discovery exits nonzero with an explicit inconclusive stderr diagnostic
  before any misleading markdown is rendered or posted.

