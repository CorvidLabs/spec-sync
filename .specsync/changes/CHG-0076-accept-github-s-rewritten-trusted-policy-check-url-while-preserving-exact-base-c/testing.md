---
change: CHG-0076-accept-github-s-rewritten-trusted-policy-check-url-while-preserving-exact-base-c
artifact: testing
---

# Testing

## Focused regressions

- Successful official check with GitHub-rewritten `/runs/<check-id>` URL passes only when one exact
  successful base-controlled workflow run exists.
- A completed parent run remains attributable after the PR head advances to its archive child.
- Wrong app, event, workflow path, repository, candidate SHA, trusted revision, PR number, status,
  conclusion, missing run, and ambiguous runs fail closed.

## Verification

- `python3 -S .github/scripts/test-verify-trusted-policy-check.py`
- `bash .github/scripts/test-lifecycle-workflows.sh`
- `specsync check --strict`
- PR #491 archive-integrity and required CI gates on the exact final child
