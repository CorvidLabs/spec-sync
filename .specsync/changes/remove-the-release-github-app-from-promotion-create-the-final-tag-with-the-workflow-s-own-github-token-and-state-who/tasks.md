---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: tasks
---

# Tasks

## Confirm the premise (before touching anything)

- [x] Re-verify `actions/variables` and `actions/secrets` are empty — the App never existed
- [x] Re-verify `environments` lists `github-pages` only — no `release` environment
- [x] Re-verify `repos/.../hooks` is `[]` — no webhook consumes a tag push
- [x] Enumerate every workflow trigger and confirm `release.yml` is the only `tags:` trigger, that
      it matches RC tags only, and that no workflow uses `workflow_run` or a `release` event
- [x] Confirm ruleset `21432148` has no `creation` rule, so `contents: write` can create the final
      tag and nothing can move it afterwards

## Workflow

- [x] Delete the `actions/create-github-app-token` step from `promote`
- [x] `permissions: contents: write` on `promote` alone; workflow default stays read-only
- [x] Delete `environment: { name: release }`
- [x] `RELEASE_TOKEN: ${{ github.token }}`; rename the guard and its error message
- [x] Keep checkout, credential helper, and the idempotent tag sequence otherwise unchanged
- [x] Rewrite the job comment: who can now mint a tag, what survives, why no environment is named,
      and why `GITHUB_TOKEN` costs nothing in triggering here
- [x] Leave no literal App variable or secret name anywhere in the file, comments included

## Validator

- [x] Add the token-identity and no-environment-gate entries to `UNENFORCED_TAG_POLICIES`
- [x] Rewrite the block comment above it to describe the decision that was taken, not a pending one

## Tests

- [x] Replace `test_promotion_uses_only_the_protected_release_app` with
      `test_promotion_mints_the_final_tag_with_the_workflow_token_alone`
- [x] Assert `contents: write` on `promote` only, exactly two in the file, none in the header
- [x] Assert no `environment:` key in `promote`
- [x] Assert no App reference anywhere in `release.yml`, comments included
- [x] Add `test_promotion_states_who_can_now_create_a_release_tag` pinning the disclosure comment
- [x] Update the `unenforced` payload assertions from two entries to three

## Docs and specs

- [x] `docs/ci-confidence.md`: three unenforced protections, the environment decision, and why
      `GITHUB_TOKEN` breaks no downstream trigger here
- [x] `specs/github/context.md`: the decision and its cost
- [x] `specs/github/requirements.md`: `REQ-github-007` acceptance criteria, plus job-scoped write
- [x] `specs/github/github.spec.md`: Invariants and Error Cases
- [x] `specs/github/tasks.md`: close the open decision task; record the optional hardening path

## Verification

- [x] `actionlint` clean on `release.yml` (shellcheck included)
- [x] `git grep SPECSYNC_RELEASE_APP\|create-github-app-token .github/` returns nothing
- [x] `python3 .github/scripts/test-validate-release-candidate.py` — 48 tests pass
- [x] `rulesets` against live payloads `21432148` / `21432132` exits 0 with three `unenforced`
- [x] Replay the `resolve` warning loop locally, positive and empty-list branches
- [x] `cargo test`
- [x] `specsync change check` and `specsync change audit --strict`

## Deliberately not done (out of scope, not pending work)

* Create the release GitHub App — the owner decided against it, and that decision is this change's
  input.
* Create the `release` environment — recorded instead as an optional hardening task in
  `specs/github/tasks.md`, with the order that makes it a real gate.
* Open a pull request or merge — branch only, per the delegating brief.
