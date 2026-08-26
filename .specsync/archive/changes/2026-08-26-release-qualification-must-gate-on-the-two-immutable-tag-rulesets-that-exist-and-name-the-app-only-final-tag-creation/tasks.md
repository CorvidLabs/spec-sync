---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: tasks
---

# Tasks

- [x] Confirm the live repository state before changing anything: two immutability rulesets present
      and active, no `SpecSync final tag creation` ruleset, no repository variables, no secrets, no
      `release` environment.
- [x] Confirm both live ruleset payloads already pass the **existing** strict validators, so the
      fix is a scope change and not a relaxation.
- [x] Drop `release_app_id` from `validate_tag_ruleset` entirely; forbid every bypass actor on both
      surviving rulesets.
- [x] Remove `validate_final_tag_creation_ruleset`, `FINAL_CREATION_RULESET_NAME`, and
      `FINAL_CREATION_RULES`.
- [x] Add `UNENFORCED_TAG_POLICIES` and emit it as `unenforced` from every successful
      `rulesets_result`.
- [x] Reduce the `rulesets` CLI to two inputs; retire `--final-creation-ruleset-json` and
      `--release-app-id` so stale callers fail loudly.
- [x] Remove the `environment` subcommand, `release_environment_result`, `ENVIRONMENT_MAX_BYTES`,
      and `ReleaseEnvironmentValidationTests`.
- [x] `release.yml`: resolve two rulesets, capture the validator result, fail if `unenforced` is
      empty, emit one `::warning::` per entry plus a step-summary section.
- [x] `release.yml`: drop the `RELEASE_APP_ID` env binding, the environments queries, and the
      now-unused `deployments: read` permission.
- [x] `release.yml`: comment `promote` so the App reads as a mechanism, not an enforced policy, and
      record that the var, secret, and environment are unset.
- [x] Rewrite `RulesetValidationTests`; add the green-run-still-declares-gaps test and the workflow
      annotation contract test.
- [x] Update `docs/ci-confidence.md` from three rulesets to two, with an explicit "not enforced"
      section.
- [x] Update `specs/github/github.spec.md` (Invariants, Error Cases) and
      `specs/github/requirements.md` (REQ-github-007) through the semantic delta.
- [x] Update `specs/github/tasks.md` so the open provisioning item matches reality.
- [x] Prove the change against the live rulesets, not fixtures: validator direct run, plus a
      verbatim replay of the whole `resolve` ruleset block against the live GitHub API.

## Follow-up, deliberately out of scope for this change

Someone must decide whether to provision the release GitHub App and the protected `release`
environment, or to retire `promote`'s App plumbing and choose another way to push a final tag.
Until that decision lands, `promote` cannot run, and RC qualification reports the gap on every run
instead of failing on it. Tracked as an open item in `specs/github/tasks.md`, not here — this
change is complete without it.
