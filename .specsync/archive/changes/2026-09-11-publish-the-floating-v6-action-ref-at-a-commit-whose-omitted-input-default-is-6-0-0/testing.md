---
change: publish-the-floating-v6-action-ref-at-a-commit-whose-omitted-input-default-is-6-0-0
artifact: testing
---

# Testing

- `git show v6:action.yml` version default is `6.0.0`.
- `git rev-parse v6^{commit}` is `ba0df69360dadd20062bcb6d2ed9557ea8964ed5`, not the `v6.0.0` commit.
- `python3 -S .github/scripts/validate-release-version.py` passes.
- `specsync check --strict github` passes.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-github-002 | Annotated `refs/tags/v6` exists. `git show v6:action.yml` default is `6.0.0`. That commit is not `v6.0.0`. Docs no longer claim the floating tag is unpublished. YAML examples remain `@v6.0.0` with `version: '6.0.0'`. |
