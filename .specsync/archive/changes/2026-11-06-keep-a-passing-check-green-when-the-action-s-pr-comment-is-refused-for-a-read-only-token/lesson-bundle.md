# Lesson bundle: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep a passing check green when the Action's PR comment is refused for a read-only token
- **Kind**: BugFix
- **Specs**: github
- **Paths**: action.yml, tests/integration/config.rs, site/src/content/docs/integrations/github-action.md, CHANGELOG.md, hi/ci.md, INTENT.md
- **Acceptance**: With comment: true and a token GitHub refuses (HTTP 403) on the comment lookup, create or update, the Post PR Comment step exits 0, emits a ::warning naming the missing pull-requests: write permission, and writes the comment markdown to GITHUB_STEP_SUMMARY (CI-3.b)
- **Acceptance**: Any other gh error in the comment step still fails the step
- **Acceptance**: The posting and updating paths with a writable token are unchanged
- **Acceptance**: A behavioural test runs the step script from action.yml against a fake gh for 403 on each call, a 500, and success; it fails before the fix
- **Acceptance**: The github-action docs comment row and CHANGELOG say a read-only token degrades to a warning plus job summary

## Evidence

- Verification commit: `564adc3168af7a8676513ccf51443f425bcbc4da`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

Issue #886, found by hosted audit S10 (2026-11-05). With `comment: true` and a read-only `GITHUB_TOKEN` (fork pull requests, or repositories set to `default_workflow_permissions: read`), the Action's Post PR Comment step calls `gh api` under `set -euo pipefail`. Creating or updating the comment returns HTTP 403, and the passing check turns red. A probe showed the lookup `GET` succeeds with `pull-requests: read`, and the create is refused even with `issues: read`.

Leif approved intent CI-3.b and this scope on 2026-11-05 at 20:30 MST, relayed by orc t18.

## From the change's design.md

# Design

The comment step wraps each `gh api` call (lookup, update, create) in a function that captures stderr. When the call fails and its stderr contains `(HTTP 403)`, the step:

1. emits `::warning title=SpecSync comment::Cannot post the PR comment: the token lacks pull-requests: write (fork pull requests get a read-only token). The check result is unchanged.`,
2. appends the comment markdown to `$GITHUB_STEP_SUMMARY` when that is set, and
3. exits 0.

Any other failure prints `gh`'s stderr and exits with `gh`'s status, as before. The success paths and their notices are unchanged. Nothing else in `action.yml` changes: the download, checksum, check and archive gate are untouched.

## From the change's testing.md

# Testing

- `config::a_comment_the_token_may_not_post_leaves_the_check_result_unchanged` (new) extracts the Post PR Comment `run:` script from `action.yml` and runs it in bash against a fake `gh`:
  - a 403 on the lookup, create or update must exit 0, print a `::warning` naming `pull-requests: write`, and write the markdown to `GITHUB_STEP_SUMMARY`;
  - a 500 must still fail the step;
  - success must still print `Posted SpecSync PR comment`.
  It fails before the fix: a 403 on the lookup exits nonzero.
- The existing `action_*` hardening tests in `tests/integration/config.rs` still pass.
- `fledge lanes run verify` before review.

## Results

- `config::a_comment_the_token_may_not_post_leaves_the_check_result_unchanged` failed first, committed as 680d9fe1: a 403 on the lookup exited nonzero. It now passes for a 403 on the lookup, create and update, for a 500 (still fails the step), and for posting and updating with a writable token.
- The first fix still failed the test. The lookup ran inside `$(...)`, so the refusal's `exit 0` ended only the subshell and the warning was captured as output. The lookup now writes to a file, and the warning goes to the step's own stdout on fd 3.
- The existing `config::action_*` hardening tests pass.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-024 | `config::a_comment_the_token_may_not_post_leaves_the_check_result_unchanged` (403 on lookup, create and update; 500; post; update) |

## Where these lessons go

- `specs/github/github.3md`
