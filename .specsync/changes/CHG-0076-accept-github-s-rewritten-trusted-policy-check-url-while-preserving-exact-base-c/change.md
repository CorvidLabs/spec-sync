---
id: CHG-0076-accept-github-s-rewritten-trusted-policy-check-url-while-preserving-exact-base-c
state: implementing
type: bug_fix
base_commit: 505b01d9b919b0aa18516961e67c1a52ab88e815
---

# Accept GitHub's rewritten trusted-policy check URL while preserving exact base-controlled workflow provenance

## Intent

Accept GitHub's rewritten trusted-policy check URL while preserving exact base-controlled workflow provenance

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The verifier accepts the successful official GitHub Actions trusted-policy check even when GitHub rewrites its details URL, independently authenticates one successful base-controlled pull_request_target workflow run by exact candidate SHA, repository, workflow path, trusted revision, and PR number, and rejects mismatched app, event, path, repository, candidate, revision, PR, unsuccessful, missing, or ambiguous provenance. Focused fixtures reproduce the rewritten URL and moved-PR-tip behavior.

## No-spec Rationale

This corrects the protected verifier to satisfy the existing trusted-policy contract; it does not change the public CI policy or lifecycle workflow.
