---
change: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token
artifact: docs
---

# Docs

- `site/src/content/docs/integrations/github-action.md`: the `comment` input row and the PR Comments section say that a token without `pull-requests: write`, as on fork pull requests, gets a warning and a job-summary copy instead of a failed check.
- `CHANGELOG.md` Unreleased: a Fixed entry for #886.
