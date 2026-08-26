---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: design
---

# Design

## Validator: `rulesets`

Before → after:

| | Before | After |
|---|--------|-------|
| Inputs | `--final-creation-ruleset-json`, `--final-immutability-ruleset-json`, `--rc-ruleset-json`, `--release-app-id` | `--final-immutability-ruleset-json`, `--rc-ruleset-json` |
| Bypass policy | one ruleset admitted exactly one `Integration` bypass actor matching the app id; two admitted none | **no ruleset admits any bypass actor** |
| Output | `final_creation`, `final_immutability`, `rc`, `mode`, `valid` | `final_immutability`, `rc`, `mode`, `unenforced`, `valid` |

`validate_tag_ruleset` loses its `release_app_id: int | None` parameter outright rather than
defaulting it to `None`. The App-only creation policy was the single place a bypass actor was ever
expected; with it gone, there is no longer a parameter through which one could be re-admitted.
Immutability that someone may bypass is not immutability.

Everything else in `validate_tag_ruleset` is untouched: exact-name, `target=tag`,
`source_type=Repository`, `enforcement=active`, exact include/exclude lists, exact rule-type set,
no duplicates, no unknown JSON fields, bounded size, no symlinked input. No broadening was
introduced to make the live configuration pass — the live payloads already satisfied every one of
these checks before the change.

`--release-app-id` and `--final-creation-ruleset-json` are removed from argparse rather than
accepted-and-ignored, so a stale caller fails with `unrecognized arguments` instead of believing
the App policy is still being checked.

## Validator: `environment`

Removed entirely, with `release_environment_result`, `ENVIRONMENT_MAX_BYTES`, and its test class.
Its only caller was `resolve`, and the environment it validated does not exist. An uninvoked CLI
subcommand describing a policy nobody enforces is exactly the drift this project exists to catch,
so it goes rather than lingering as dead plumbing.

## The visibility mechanism

`UNENFORCED_TAG_POLICIES` is a module constant listing, in prose a human can act on, each tag
protection the design once specified and this repository does not have. `rulesets_result` copies
it into every successful result as `unenforced`.

`release.yml` then:

1. captures the validator's JSON to `$rulesets_result_file`;
2. **fails the job** if `.unenforced | length` is `< 1` — the admission cannot be silently emptied
   into a clean bill of health;
3. emits one `::warning title=Release protection not enforced::` annotation per entry, and appends
   each to `$GITHUB_STEP_SUMMARY` under a heading.

Warnings, not notices: these appear on the run summary page where someone judging a release will
see them. The failure-if-empty step is what makes this a mechanism rather than a comment.

## `SPECSYNC_RELEASE_APP_ID` plumbing

Removed where it became dead: the `RELEASE_APP_ID` env binding in `resolve` and the
`--release-app-id` validator input. Kept where it is still load-bearing: `promote`'s
`actions/create-github-app-token` step, which mints the repository-scoped token that pushes the
final tag. A workflow comment on `promote` records that the App is now a *mechanism*, not a policy,
and that the var, the secret, and the `release` environment are all currently unset — so nobody
reads that job's silence as readiness.

`permissions: deployments: read` is dropped from the workflow's top level; its only consumer was
the deleted environments query.
