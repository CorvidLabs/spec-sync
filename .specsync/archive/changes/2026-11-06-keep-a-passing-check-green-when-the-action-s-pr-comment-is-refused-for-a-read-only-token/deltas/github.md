## ADDED

### REQUIREMENT REQ-github-024

When the Action's `comment` input is true and GitHub refuses its token (HTTP 403) on the PR comment
lookup, create or update, the Post PR Comment step SHALL NOT fail. It SHALL emit a warning naming
the missing `pull-requests: write` permission and that fork pull requests get a read-only token,
write the comment markdown to the job summary, and exit 0, so the check's result is the spec check's
result (CI-3.b). Any other failure of the comment step SHALL still fail it.

Acceptance Criteria
- A 403 on the comment lookup, create or update ends the step with exit 0, a `::warning` naming
  `pull-requests: write`, and the comment markdown in `GITHUB_STEP_SUMMARY`.
- A non-403 `gh` error, such as HTTP 500, still fails the step.
- With a writable token the step posts a new comment or updates the existing one, as before.
