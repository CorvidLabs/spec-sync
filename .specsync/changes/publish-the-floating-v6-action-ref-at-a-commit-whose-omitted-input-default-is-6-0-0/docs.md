---
change: publish-the-floating-v6-action-ref-at-a-commit-whose-omitted-input-default-is-6-0-0
artifact: docs
---

# Docs

- `action.yml` description names `@v6` as using the 6.0.0 default.
- README, SECURITY.md, ADOPTING.md, RELEASING.md, github-action.md, and CHANGELOG Unreleased stop
  saying the floating tag does not exist.
- RELEASING.md forbids retargeting `v6` at `v6.0.0`.
- YAML Action examples remain `uses: CorvidLabs/spec-sync@v6.0.0` with `version: '6.0.0'`.
- Canonical `specs/github` purpose, REQ-github-002, testing.md, context.md, and changelog v33.
