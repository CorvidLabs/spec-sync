---
change: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token
artifact: design
---

# Design

The comment step wraps each `gh api` call (lookup, update, create) in a function that captures stderr. When the call fails and its stderr contains `(HTTP 403)`, the step:

1. emits `::warning title=SpecSync comment::Cannot post the PR comment: the token lacks pull-requests: write (fork pull requests get a read-only token). The check result is unchanged.`,
2. appends the comment markdown to `$GITHUB_STEP_SUMMARY` when that is set, and
3. exits 0.

Any other failure prints `gh`'s stderr and exits with `gh`'s status, as before. The success paths and their notices are unchanged. Nothing else in `action.yml` changes: the download, checksum, check and archive gate are untouched.
