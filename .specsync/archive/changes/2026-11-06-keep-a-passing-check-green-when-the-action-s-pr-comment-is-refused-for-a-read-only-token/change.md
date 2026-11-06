---
id: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token
state: archived
type: bug_fix
base_commit: ad50bdef3db58a715877877d2407485bcaed0f32
---

# Keep a passing check green when the Action's PR comment is refused for a read-only token

## Intent

Keep a passing check green when the Action's PR comment is refused for a read-only token

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- With comment: true and a token GitHub refuses (HTTP 403) on the comment lookup, create or update, the Post PR Comment step exits 0, emits a ::warning naming the missing pull-requests: write permission, and writes the comment markdown to GITHUB_STEP_SUMMARY (CI-3.b)
- Any other gh error in the comment step still fails the step
- The posting and updating paths with a writable token are unchanged
- A behavioural test runs the step script from action.yml against a fake gh for 403 on each call, a 500, and success; it fails before the fix
- The github-action docs comment row and CHANGELOG say a read-only token degrades to a warning plus job summary

## No-spec Rationale

Not applicable
