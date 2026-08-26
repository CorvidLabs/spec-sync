# Lesson bundle — release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Release qualification must gate on the two immutable tag rulesets that exist and name the App-only final-tag creation policy it no longer enforces
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/workflows/release.yml, .github/scripts/validate-release-candidate.py, .github/scripts/test-validate-release-candidate.py, docs/ci-confidence.md, specs/github/requirements.md, specs/github/github.spec.md, specs/github/tasks.md
- **Acceptance**: The resolve job of .github/workflows/release.yml resolves and validates exactly two repository tag rulesets — 'SpecSync immutable RC tags' and 'SpecSync immutable final tags' — and no longer resolves 'SpecSync final tag creation', queries the release deployment environment, or reads vars.SPECSYNC_RELEASE_APP_ID. The validator's 'rulesets' command accepts only --final-immutability-ruleset-json and --rc-ruleset-json, rejects --release-app-id and --final-creation-ruleset-json, and emits a non-empty 'unenforced' array naming the App-only final-tag creation policy and the unverified protected release environment. Every release run prints those unenforced items as GitHub warning annotations so a green run cannot be read as proof that App-only final-tag creation is enforced. Both immutability rulesets stay strict: any bypass actor, broadened include/exclude pattern, extra or missing rule type, inactive enforcement, or non-Repository source is still rejected. The 'environment' subcommand is removed with its tests. docs/ci-confidence.md, specs/github/requirements.md, specs/github/github.spec.md and specs/github/tasks.md describe two enforced rulesets and explicitly record the dropped App-only creation policy and the unverified release environment. python3 .github/scripts/test-validate-release-candidate.py passes, and running the 'rulesets' command against the live payloads of rulesets 21432132 and 21432148 exits 0.

## Evidence

- Verification commit: `c61bd1ec334bf917a15132855690eaa8c39681db`
- Base commit: `e82542d19ce8d79926b144a0e38d4d620b120715`
- Verified by: `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

`release.yml` has failed on every RC tag since the ruleset check landed in #492 (2026-08-01):
`v6.0.0-rc.1` through `rc.6`, six consecutive candidates. `v5.2.0` (2026-07-19) shipped only
because the check did not exist yet. The workflow has **never** passed with this check in place.

The `resolve` job demanded three repository tag rulesets plus `vars.SPECSYNC_RELEASE_APP_ID`:

| Ruleset | Needs an App? | Exists? |
|---------|---------------|---------|
| `SpecSync final tag creation` — blocks creation of `refs/tags/v*.*.*` (excluding rc) except for one bypass actor, the release App | yes | **no** |
| `SpecSync immutable final tags` — update+deletion, validated with `release_app_id=None` | no | yes (id 21432148) |
| `SpecSync immutable RC tags` — update+deletion on `refs/tags/v*.*.*-rc.*` | no | yes (id 21432132) |

Then it demanded a protected `release` deployment environment.

Verified against the live repository on 2026-08-25:

- `gh api repos/CorvidLabs/spec-sync/rulesets` returns the two immutability rulesets, both
  `target=tag`, `source_type=Repository`, `enforcement=active`, `bypass_actors=[]`, and no
  `SpecSync final tag creation` ruleset.
- `gh api repos/CorvidLabs/spec-sync/actions/variables` → `total_count: 0`. There is no
  `SPECSYNC_RELEASE_APP_ID`.
- `gh api repos/CorvidLabs/spec-sync/actions/secrets` → `total_count: 0`. There is no
  `SPECSYNC_RELEASE_APP_PRIVATE_KEY`.
- `gh api repos/CorvidLabs/spec-sync/environments` returns `github-pages` only. There is no
  `release` environment, so the `environment` check would 404 under `set -euo pipefail` even if
  the ruleset check were fixed.

So the release App was never provisioned. `--release-app-id "$RELEASE_APP_ID"` expanded to
`--release-app-id ""`, which argparse rejects before any ruleset is read — the two rulesets that
DO exist were never validated even once. That is the trap this repo exists to catch: a gate that
always fails proves less than no gate at all, because it hides the checks behind it.

## Decision taken

The repository owner decided to adopt the two App-free rulesets and skip the App-only creation
policy. The two rulesets were created and are live and active before this change was written.

The constraint this change is built around: **dropping a protection is allowed; dropping it
quietly is not.** So the validator refuses to succeed without emitting a non-empty `unenforced`
list, and `release.yml` prints every entry as a `::warning::` annotation and into the step summary
on every run. A green release cannot be read as evidence that App-only final-tag creation is
enforced, because the run itself says it is not.

## Already ruled out

- **Creating the App / the `release` environment.** Out of scope and not the owner's decision.
  `promote` keeps `vars.SPECSYNC_RELEASE_APP_ID` — the App is how promotion *pushes* a tag, not a
  policy stopping anyone else from creating one. Promotion stays unprovisioned and fails closed.
- **Switching `promote` to push with `GITHUB_TOKEN` + `contents: write`.** That would grant the
  default workflow token tag-write on every promote run. A larger security decision than the one
  delegated, and not needed to unblock RC qualification.
- **Relaxing the two surviving rulesets.** The validator deliberately refuses supersets, and both
  live payloads already pass its existing strict checks unmodified — confirmed before editing
  anything. No strictness was traded away to make this pass.

## From the change's design.md

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

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-007` | `RulesetValidationTests` proves the two-ruleset contract, that no bypass actor is admissible on either, and that no broadening of name/target/source/enforcement/ref-patterns/rule-types is accepted; `test_successful_rulesets_declare_every_unenforced_tag_protection` and `test_release_announces_every_unenforced_tag_protection_on_every_run` prove a green run still states both unenforced protections and that an empty statement fails the job; `test_release_queries_and_validates_exactly_the_two_immutability_rulesets` proves the App-only creation policy and the `release` environment query are gone from the workflow. Live proof: the `rulesets` command exits 0 against the actual payloads of rulesets 21432148 and 21432132, and the whole `resolve` ruleset block replays against the live GitHub API. |

## Automated

`python3 .github/scripts/test-validate-release-candidate.py` (wired through
`fledge.toml` `[tasks.release-candidate-test]`, the `verify` lane, and the RC lane). 47 tests pass.

`RulesetValidationTests` — rewritten for two rulesets:

- `test_rulesets_cli_accepts_exact_active_repository_policies` — both fixtures validate; output has
  no `final_creation` key; `--help` no longer advertises `--final-creation-ruleset-json` or
  `--release-app-id`.
- `test_successful_rulesets_declare_every_unenforced_tag_protection` — a **green** run still emits
  two `unenforced` entries naming `SpecSync final tag creation`, the release GitHub App, and the
  `'release' deployment environment`; each contains `NOT`. This is the regression guard against
  the change being quietly re-weakened into a silent pass.
- `test_rulesets_cli_requires_both_inputs_and_rejects_retired_flags` — both inputs required;
  `--release-app-id` and `--final-creation-ruleset-json` now fail with `unrecognized arguments`;
  the `ruleset` and `environment` subcommands both fail with `invalid choice`.
- `test_immutability_rulesets_reject_every_bypass_actor` — extended to `Integration`, `User`,
  `RepositoryRole`, and `OrganizationAdmin` on **both** rulesets. Previously `Integration` was
  admissible on the creation policy; now no actor type is admissible anywhere.
- Broadening guards retained unchanged: wrong name/target/source_type/enforcement, broadened or
  duplicated include/exclude patterns, unknown condition fields, missing/extra/duplicate/
  parameterized rule types, unknown top-level fields, duplicate JSON keys, non-object payloads,
  oversized payloads, duplicate ruleset ids, symlinked inputs.

`WorkflowSourceContractTests`:

- `test_release_queries_and_validates_exactly_the_two_immutability_rulesets` — exactly two
  `resolve_ruleset` calls; asserts the whole workflow file no longer contains
  `resolve_ruleset "SpecSync final tag creation"`, `--final-creation-ruleset-json`,
  `--release-app-id`, `"repos/${REPOSITORY}/environments/release"`,
  `validate-release-candidate.py environment`, or the `RELEASE_APP_ID` env binding.
- `test_release_announces_every_unenforced_tag_protection_on_every_run` — asserts the result file
  capture, the `< 1` fail-closed guard and its error text, the `::warning::` line, the
  `jq -r '.unenforced[]'` loop, and two `$GITHUB_STEP_SUMMARY` writes.
- `test_promotion_uses_only_the_protected_release_app` — unchanged and still passing; the App
  remains promotion's push mechanism.

## Live verification (cannot be covered by fixtures)

A tag cannot be pushed to test this, so the validator was run against the **actual** live payloads
rather than fixtures:

```
gh api "repos/CorvidLabs/spec-sync/rulesets/21432148?includes_parents=true" > final-immutability-ruleset.json
gh api "repos/CorvidLabs/spec-sync/rulesets/21432132?includes_parents=true" > rc-ruleset.json
python3 .github/scripts/validate-release-candidate.py rulesets \
  --final-immutability-ruleset-json final-immutability-ruleset.json \
  --rc-ruleset-json rc-ruleset.json
```

Exit 0. Also run before editing the validator: both live payloads already passed
`validate_rc_tag_ruleset` and `validate_final_tag_immutability_ruleset` unmodified, proving no
strictness was relaxed to accommodate the live configuration.

The full `resolve` ruleset block was then replayed verbatim as a shell script against the live
GitHub API — `gh api .../rulesets`, the `resolve_ruleset` helper, the validator call, the
empty-`unenforced` guard, the warning loop, and the step-summary writes — exiting 0 and printing
both warnings. Only `mapfile` was shimmed, because macOS ships bash 3.2 while runners use bash 5.

## Not verified

Nothing exercises `promote`, `build`, or `release` end to end: promotion requires a release App
that does not exist. Those jobs are unreachable on the RC-push path this change fixes and were not
altered beyond a comment.

## Where these lessons go

- `specs/github/context.md`
