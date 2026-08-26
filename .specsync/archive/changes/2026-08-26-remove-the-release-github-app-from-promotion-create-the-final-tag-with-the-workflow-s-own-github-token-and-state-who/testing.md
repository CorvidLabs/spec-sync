---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-007` | `test_promotion_mints_the_final_tag_with_the_workflow_token_alone` proves promotion creates the final tag with `${{ github.token }}`, that `contents: write` appears on `promote` and exactly twice in the whole workflow, that the workflow header stays read-only, that no `environment:` key remains on the job, and that no App step, variable, secret, or token output survives; `test_promotion_states_who_can_now_create_a_release_tag` proves the workflow itself still says who can now mint a release tag and why no environment is named; `test_release_queries_and_validates_exactly_the_two_immutability_rulesets` proves no App reference remains anywhere in `release.yml`, comments included; `test_successful_rulesets_declare_every_unenforced_tag_protection` and `test_release_announces_every_unenforced_tag_protection_on_every_run` prove a green run states all three unenforced protections and that an empty statement fails the job; `RulesetValidationTests` proves both immutability rulesets are still validated strictly with no bypass actor and no broadening. Live proof, since a final tag cannot be pushed: `rulesets` exits 0 against the actual payloads of 21432148 and 21432132 and emits three `unenforced` entries, and the `resolve` disclosure block replays verbatim against them on both its positive and empty-list branches. `actionlint` (shellcheck included) reports no issue in `release.yml`; 48 validator tests and 399 cargo tests pass. |

A final tag cannot be pushed to test, and `promote` has never run — there is no green baseline to
compare against. So the evidence is: the workflow parses and lints, the source contract is pinned by
tests, the disclosure block was replayed against live data, and nothing else regressed.

## What is pinned by tests

`.github/scripts/test-validate-release-candidate.py`:

| Test | Pins |
|------|------|
| `test_promotion_mints_the_final_tag_with_the_workflow_token_alone` | `RELEASE_TOKEN: ${{ github.token }}`; `contents: write` once in `promote` and exactly twice in the file; no `contents: write` in the workflow header; no `environment:` key; no `create-github-app-token`, `SPECSYNC_RELEASE_APP_ID`, `SPECSYNC_RELEASE_APP_PRIVATE_KEY`, `permission-contents: write`, or `release-app-token`; `persist-credentials: false`; the three `git_release` calls; no `git push origin`, no `git@github.com`, no `SPECSYNC_RELEASE_TAG_KEY` |
| `test_promotion_states_who_can_now_create_a_release_tag` | The disclosure comment survives: `WHO CAN MINT A RELEASE TAG`, `THE PROTECTION THAT WAS GIVEN UP`, ``NO `environment:` HERE, DELIBERATELY``, and the sentence naming who can now run the lane |
| `test_release_queries_and_validates_exactly_the_two_immutability_rulesets` | Extended retired-strings list now asserts no App reference anywhere in `release.yml`, comments included |
| `test_successful_rulesets_declare_every_unenforced_tag_protection` | Three `unenforced` entries; each contains `NOT`; the joined text names `SpecSync final tag creation`, `release GitHub App`, `GITHUB_TOKEN`, and `deployment-environment gate` |
| `test_release_announces_every_unenforced_tag_protection_on_every_run` (unchanged) | The `resolve` warning loop, the empty-list failure, and both step-summary writes |

The count assertion (`contents: write` exactly twice) is the one that catches the likeliest future
mistake: a third job quietly acquiring ref-write.

## Results

| Check | Result |
|-------|--------|
| `actionlint` on `release.yml` | clean, shellcheck included |
| `actionlint` on all workflows | only a pre-existing `SC2012` in `rc-assets.yml`, untouched by this change |
| `git grep "SPECSYNC_RELEASE_APP\|create-github-app-token" .github/` | no matches |
| `python3 .github/scripts/test-validate-release-candidate.py` | **48 tests, OK** |
| `cargo test --release` | **399 passed, 0 failed, 6 ignored** |
| `specsync change check` (full suite, nothing ignored) | **405 passed, 0 failed** |
| `specsync check --strict --require-coverage 100` | 62 specs, 0 failed; file and LOC coverage 100% |
| `cargo fmt --check` | clean |

## Live replay, since a tag cannot be pushed

`rulesets` run against the real payloads of `21432148` and `21432132`
(`gh api repos/CorvidLabs/spec-sync/rulesets/<id>?includes_parents=true`) exits **0**, still reports
both rulesets `valid: true` with `bypass_actors: []` and rules `["update", "deletion"]`, and emits
three `unenforced` entries.

The `resolve` disclosure block was then replayed verbatim — same `jq` expressions, same
`RUNNER_TEMP`/`GITHUB_STEP_SUMMARY` handling, same failure branch — against those payloads:

```
::warning title=Release protection not enforced::Final-tag creation is NOT restricted to a release GitHub App…
::warning title=Release protection not enforced::The final tag is minted by this workflow's own GITHUB_TOKEN, NOT by a separate release identity…
::warning title=Release protection not enforced::Promotion is NOT behind a deployment-environment gate…
---- unenforced_count=3 ----
### Release tag protections NOT verified by this run
- :warning: (all three, repeated into the step summary)
NEGATIVE BRANCH OK: empty unenforced list would fail the step
```

Both branches were exercised: three entries pass and print; an empty list takes the `exit 1` path.

## Live preconditions re-verified rather than assumed

- `actions/variables` → `total_count: 0`; `actions/secrets` → `total_count: 0`
- `environments` → `github-pages` only
- `hooks` → `[]`
- `release.yml` is the only workflow with a `tags:` trigger, matching RC tags only; no
  `workflow_run`, `repository_dispatch`, or `release` trigger anywhere
- ruleset `21432148` has no `creation` rule, so `contents: write` can create the final tag and
  nothing can move it afterwards

## What could NOT be tested

- **An actual promotion.** `promote` has never run and a final tag cannot be pushed from here. The
  first real execution is the first proof that `GITHUB_TOKEN` pushes the tag successfully. The
  push path itself — remote URL, credential helper, `x-access-token` username, idempotent
  `ls-remote`/`fetch`/compare branch — is byte-for-byte what was already there; only the credential
  changed.
- **GitHub's behaviour on an auto-created environment.** The claim that a referenced environment is
  materialized without protection rules is GitHub's documented behaviour, not something this change
  observed. It is the reason the reference was dropped; if it were wrong, the reference would be
  harmless rather than helpful, so the decision is safe either way.
- **That no external GitHub App installation reacts to a final-tag push.** Repository webhooks are
  empty and no workflow listens, but installed-App event subscriptions are not readable with the
  available credentials. Nothing in this repository configures such an integration.
