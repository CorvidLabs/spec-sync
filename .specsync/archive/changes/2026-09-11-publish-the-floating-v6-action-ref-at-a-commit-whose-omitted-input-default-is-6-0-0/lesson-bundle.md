# Lesson bundle — publish-the-floating-v6-action-ref-at-a-commit-whose-omitted-input-default-is-6-0-0

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Publish the floating v6 Action ref at a commit whose omitted-input default is 6.0.0
- **Kind**: Documentation
- **Specs**: github
- **Paths**: action.yml, site/src/content/docs/integrations/github-action.md, README.md, SECURITY.md
- **Acceptance**: GitHub ref refs/tags/v6 exists and is annotated.
- **Acceptance**: git show v6:action.yml version default is 6.0.0.
- **Acceptance**: v6 does not resolve to tag v6.0.0 (that tag still defaults to 6.0.0-rc.14).
- **Acceptance**: README, SECURITY.md, ADOPTING.md, RELEASING.md, and github-action.md no longer say the floating v6 tag does not exist.
- **Acceptance**: YAML Action examples remain uses @v6.0.0 with version 6.0.0.
- **Acceptance**: specsync check --strict github passes.

## Evidence

- Verification commit: `6920ff095227aa8c87ff0b3022d9aae9bf5b0d1d`
- Base commit: `ba0df69360dadd20062bcb6d2ed9557ea8964ed5`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

Immutable tag `v6.0.0` cannot be rewritten. Its `action.yml` still defaults omitted `version` to
`6.0.0-rc.14`. `main` already defaults to `6.0.0`. REQ-github-002 requires a floating `v6`
compatibility ref whose downloadable default is the promoted stable.

Pointing `v6` at `v6.0.0` would re-ship the RC default. The floating tag is an annotated ref at
`ba0df693` (origin/main when published), whose `action.yml` default is `6.0.0`. YAML examples stay
on `@v6.0.0` with `version: '6.0.0'` because the release validator requires that exact pin.

## From the change's testing.md

# Testing

- `git show v6:action.yml` version default is `6.0.0`.
- `git rev-parse v6^{commit}` is `ba0df69360dadd20062bcb6d2ed9557ea8964ed5`, not the `v6.0.0` commit.
- `python3 -S .github/scripts/validate-release-version.py` passes.
- `specsync check --strict github` passes.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-github-002 | Annotated `refs/tags/v6` exists. `git show v6:action.yml` default is `6.0.0`. That commit is not `v6.0.0`. Docs no longer claim the floating tag is unpublished. YAML examples remain `@v6.0.0` with `version: '6.0.0'`. |

## Where these lessons go

- `specs/github/context.md`
