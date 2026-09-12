---
id: publish-the-floating-v6-action-ref-at-a-commit-whose-omitted-input-default-is-6-0-0
state: archived
type: documentation
base_commit: ba0df69360dadd20062bcb6d2ed9557ea8964ed5
---

# Publish the floating v6 Action ref at a commit whose omitted-input default is 6.0.0

## Intent

Publish the floating v6 Action ref at a commit whose omitted-input default is 6.0.0

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- GitHub ref refs/tags/v6 exists and is annotated.
- git show v6:action.yml version default is 6.0.0.
- v6 does not resolve to tag v6.0.0 (that tag still defaults to 6.0.0-rc.14).
- README, SECURITY.md, ADOPTING.md, RELEASING.md, and github-action.md no longer say the floating v6 tag does not exist.
- YAML Action examples remain uses @v6.0.0 with version 6.0.0.
- specsync check --strict github passes.

## No-spec Rationale

Not applicable
