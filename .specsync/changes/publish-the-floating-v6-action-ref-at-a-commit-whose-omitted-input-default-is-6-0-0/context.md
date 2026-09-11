---
change: publish-the-floating-v6-action-ref-at-a-commit-whose-omitted-input-default-is-6-0-0
artifact: context
---

# Context

Immutable tag `v6.0.0` cannot be rewritten. Its `action.yml` still defaults omitted `version` to
`6.0.0-rc.14`. `main` already defaults to `6.0.0`. REQ-github-002 requires a floating `v6`
compatibility ref whose downloadable default is the promoted stable.

Pointing `v6` at `v6.0.0` would re-ship the RC default. The floating tag is an annotated ref at
`ba0df693` (origin/main when published), whose `action.yml` default is `6.0.0`. YAML examples stay
on `@v6.0.0` with `version: '6.0.0'` because the release validator requires that exact pin.
