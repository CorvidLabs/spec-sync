---
change: publish-the-floating-v6-action-ref-at-a-commit-whose-omitted-input-default-is-6-0-0
artifact: requirements
---

# Requirements

## Functional

### REQ-github-002 (modified AC)

The floating `v6` ref SHALL resolve to a commit whose composite Action omitted-input default is
`6.0.0`. It SHALL NOT resolve to immutable tag `v6.0.0`.

Acceptance Criteria

- `git show v6:action.yml` version default is `6.0.0`.
- `git rev-parse v6^{commit}` is not the `v6.0.0` commit.
- `@v6.0.0` still embeds default `6.0.0-rc.14`; docs tell consumers of that tag to pass
  `version: '6.0.0'`.
- README/site YAML examples remain `@v6.0.0` with `version: '6.0.0'`.
