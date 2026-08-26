# Lesson bundle — remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Remove the release GitHub App from promotion: create the final tag with the workflow's own GITHUB_TOKEN and state who can now mint a release tag
- **Kind**: Operations
- **Specs**: github
- **Paths**: .github/workflows/release.yml, .github/scripts/validate-release-candidate.py, .github/scripts/test-validate-release-candidate.py, docs/ci-confidence.md, specs/github/github.spec.md, specs/github/requirements.md, specs/github/tasks.md, specs/github/context.md
- **Acceptance**: The promote job of .github/workflows/release.yml creates the final tag with the workflow's own GITHUB_TOKEN and nothing else: the actions/create-github-app-token step is gone, and no file under .github/ references vars.SPECSYNC_RELEASE_APP_ID, secrets.SPECSYNC_RELEASE_APP_PRIVATE_KEY, or actions/create-github-app-token. Write access is scoped to that one job — the workflow-level permissions block stays contents: read / actions: read / checks: read, and promote declares permissions: contents: write on the job alone. The 'environment: release' reference is removed, because the release environment does not exist and referencing it makes GitHub auto-create an unprotected environment that reads in the UI as a deployment gate while enforcing nothing; the workflow records that removal and what would be required to make it a real gate. The protection given up is stated where a reader will see it, not buried: validate-release-candidate.py's UNENFORCED_TAG_POLICIES says that final-tag creation is unrestricted AND that the final tag is now minted by the workflow's own token, so anyone able to dispatch release.yml from the default branch can create refs/tags/vX.Y.Z, and that no deployment-environment gate stands between a dispatch and a release tag; release.yml still fails when that list is empty and still prints every entry as a ::warning:: annotation and into the step summary on every run, green ones included. Tag immutability is unchanged: both rulesets are still validated strictly with no bypass actor. docs/ci-confidence.md, specs/github/github.spec.md, specs/github/requirements.md, specs/github/context.md and specs/github/tasks.md describe GITHUB_TOKEN-minted promotion with no App and no environment gate, and the open 'decide the fate of App-only final-tag creation' task is closed with the decision that was made. python3 .github/scripts/test-validate-release-candidate.py passes, its promote contract test asserts the GITHUB_TOKEN shape and the absence of every App and environment reference, actionlint reports no issue on release.yml, and cargo test passes.

## Evidence

- Verification commit: `c61bd1ec334bf917a15132855690eaa8c39681db`
- Base commit: `0176c6a516e03f63ea83fb401d6f934ac2800a41`
- Verified by: `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

This finishes the decision the previous change deliberately left open.

`release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation`
removed the release GitHub App from the `resolve` job and relaxed ruleset validation to the two
rulesets that actually exist. It left `promote`'s App plumbing in place on purpose, and its own
"Already ruled out" section says so:

> **Switching `promote` to push with `GITHUB_TOKEN` + `contents: write`.** That would grant the
> default workflow token tag-write on every promote run. A larger security decision than the one
> delegated, and not needed to unblock RC qualification.

It also opened a task in `specs/github/tasks.md`: *"Either provision them, or retire `promote`'s App
plumbing and choose another way to push a final tag."*

**The repository owner has now made that decision: no GitHub App.** This change carries it out.

## What was actually there

Three references to the App survived the previous change, all in `promote`, all serving one purpose
— `permission-contents: write`, to create the final tag:

```yaml
- name: Mint repository-scoped release app token
  id: release-app-token
  uses: actions/create-github-app-token@fee1f7d63c2ff003460e3d139729b119787bc349 # v2
  with:
    app-id: ${{ vars.SPECSYNC_RELEASE_APP_ID }}
    private-key: ${{ secrets.SPECSYNC_RELEASE_APP_PRIVATE_KEY }}
```

Plus `environment: { name: release }` on the job.

## Verified against the live repository, 2026-08-25

Re-checked independently rather than carried over from the previous change:

- `gh api repos/CorvidLabs/spec-sync/actions/variables` → `total_count: 0`.
- `gh api repos/CorvidLabs/spec-sync/actions/secrets` → `total_count: 0`.
  The App never existed, so `promote` has never run.
- `gh api repos/CorvidLabs/spec-sync/environments` → `github-pages` only. There is no `release`
  environment.
- `gh api repos/CorvidLabs/spec-sync/hooks` → `[]`. No repository webhook consumes a tag push.
- `.github/workflows/release.yml` is the only workflow in the repository with a `tags:` trigger,
  and it matches `v[0-9]+.[0-9]+.[0-9]+-rc.[0-9]+` — RC tags only. `ci.yml`, `pages.yml`, and
  `trust.yml` trigger on branch pushes and pull requests; `rc-assets.yml` is `workflow_dispatch`
  only. No workflow uses `workflow_run` or a `release` event.
- Ruleset `21432148` (`SpecSync immutable final tags`) has rules `update` and `deletion` only, and
  `bypass_actors: []`. It does **not** restrict creation, so a `GITHUB_TOKEN` with
  `contents: write` can create `refs/tags/vX.Y.Z`, and nobody — that token included — can move or
  delete it afterwards.

The last two matter together: the usual objection to `GITHUB_TOKEN` is that pushes made with it do
not trigger other workflows. Here nothing listens for a final tag, so that objection has nothing to
break. It becomes a real constraint only if something later needs to react to `vX.Y.Z`; such work
must be called from inside `release.yml` rather than triggered by the tag, and the workflow says so
at the `promote` job.

## The decision, and what it costs

`promote` creates the final tag with the workflow's own `GITHUB_TOKEN`, under `contents: write`
declared on that job alone. The workflow-level default stays `contents: read` / `actions: read` /
`checks: read`, so no other job in the lane gains ref-write.

The authority that was given up is real and is stated in three places rather than assumed:

1. **At the job.** A comment block headed `WHO CAN MINT A RELEASE TAG` names the loss in the file a
   reader auditing promotion actually opens.
2. **In every run.** `UNENFORCED_TAG_POLICIES` gained two entries — that the tag is minted by the
   workflow's own token rather than a separate identity, and that no deployment-environment gate
   stands in between. `release.yml` still fails if that list is empty and still prints every entry
   as a `::warning::` annotation and into the step summary on green runs.
3. **In the docs.** `docs/ci-confidence.md` "Tag authority" now lists three unenforced protections,
   not two.

The loss in one sentence: **running the release lane and holding release authority are now the same
permission.** Anyone who can dispatch `release.yml` from the default branch can cause a release tag
to be created. An App key is the one credential a workflow author cannot reach by editing the
workflow; there no longer is one.

What survives: `promote` still `needs: [resolve, validate, authorize-release]`, so a tag from that
job still follows a candidate qualified on Ubuntu, macOS, and Windows. And both immutability
rulesets still admit no bypass actor, so a shipped `vX.Y.Z` is still permanent. Immutability was
always the protection that mattered most and it is untouched.

## The `environment: release` reference

Dropped, not kept-with-a-comment. `promote` named an environment that has never existed. GitHub
materializes a referenced environment on first use **with no protection rules**, so the reference
would have added a `release` entry to the repository's Environments and Deployments UI that gates
nothing while looking like a gate — strictly worse than claiming no gate at all, because only one
of the two can mislead someone auditing the release path.

Keeping it with an explanatory comment was considered and rejected: the comment lives in the
workflow, the misleading artifact lives in the GitHub UI, and the two audiences do not overlap.

The path back is written down instead of implied, in `specs/github/tasks.md` and at the job:
create the environment **with** required reviewers and a `main`-only deployment branch policy
first, then re-add `environment: release`, then restore a qualification check that proves those
rules are still in place.

## Already ruled out

- **Provisioning the App after all.** That is the decision that was made, and it was made the other
  way.
- **A deploy key or PAT instead.** Both reintroduce a long-lived credential to rotate, and neither
  narrows who can cause a tag: a workflow author reaches a repository secret exactly as easily as
  they reach `GITHUB_TOKEN`. Only an environment with required reviewers narrows that, and that is
  recorded as the optional hardening step.
- **Leaving `environment: release` in place "for later".** See above; an auto-created environment
  has no protection rules.
- **Weakening either immutability ruleset.** Untouched. `rulesets` still rejects any bypass actor,
  broadened pattern, extra rule, or inactive enforcement, and both live payloads still validate.

## From the change's design.md

# Design

## Shape of the change

```
                       BEFORE                                    AFTER
  workflow permissions: contents: read                  workflow permissions: contents: read
                                                                  (unchanged)
  promote:                                              promote:
    environment: { name: release }   <- absent env        (no environment)
    permissions: contents: read                           permissions: contents: write
    steps:                                                steps:
      - create-github-app-token      <- unprovisioned       - checkout (persist-credentials: false)
          app-id:    vars.…APP_ID                           - tag + push via credential helper
          private-key: secrets.…KEY                             using ${{ github.token }}
          permission-contents: write
      - checkout (persist-credentials: false)
      - tag + push via credential helper
          using steps.release-app-token.outputs.token
```

Everything below the token source is deliberately unchanged: the same `persist-credentials: false`
checkout, the same one-remote credential helper, the same idempotent
`ls-remote` → `fetch` → compare → else `tag -a` + `push` sequence, the same annotated tag message.
Only the credential changes.

## Why the credential helper stays

`persist-credentials: false` on checkout means no token is written into `.git/config`. The push then
authenticates through a `credential.helper` that exists only for the duration of the `git` process
and only for the release remote. That property is worth as much with `GITHUB_TOKEN` as it was with
an App token — arguably more, since `GITHUB_TOKEN` is present in the job environment anyway — so the
mechanism is kept verbatim and the comment above it now says why.

`x-access-token` remains the correct username for `GITHUB_TOKEN` over HTTPS.

## Why `contents: write` on the job, never the workflow

Job-level `permissions:` replaces the workflow-level map rather than merging. `promote` makes no
`gh api` call and needs no `actions:`/`checks:` scope, so `contents: write` alone is both sufficient
and minimal. Declaring it workflow-wide would hand ref-write to the other six jobs in the file — `resolve`,
`validate`, `qualify`, `record-qualification`, `authorize-release`, and `build`, several of which
check out an operator-supplied ref — for no reason.

After this change the workflow contains exactly two `contents: write` grants: `promote` and
`release`. A test pins that count so a third cannot appear unnoticed.

## Why the environment reference is deleted rather than annotated

Two audiences see two different artifacts. A workflow comment reaches the person reading
`release.yml`. An auto-created `release` environment reaches the person reading the repository's
Environments and Deployments pages, and it tells them a deployment gate exists. Since GitHub
materializes a referenced environment with **no** protection rules, retaining the reference would
have created the misleading artifact for the second audience on the first promotion, where no
comment can reach them.

Deleting the reference makes the two audiences agree: there is no gate, and neither surface claims
one. The route to a real gate is written at the job and in `specs/github/tasks.md`, in the order
that matters — environment with rules **first**, reference **second**, proving check **third**.

## Why the disclosure has three homes

The `unenforced` list is emitted per run; the workflow comment is read at audit time; the doc is
read when someone asks what the release lane guarantees. A reader of any one of the three must not
be able to conclude that a release tag implies an authority that does not exist. The list is the
enforced one — `release.yml` fails when it is empty — and the other two are prose that a test pins
by keyword (`WHO CAN MINT A RELEASE TAG`, `THE PROTECTION THAT WAS GIVEN UP`,
`NO \`environment:\` HERE, DELIBERATELY`).

Splitting the loss into two `unenforced` entries rather than extending the existing one is
deliberate: "creation is unrestricted" and "the release lane *is* the release authority" are
different facts, and a reader told only the first would not learn the second.

## Rejected alternatives

| Alternative | Why not |
|-------------|---------|
| Provision the App | The owner decided against it; that decision is the input to this change |
| Deploy key or PAT | Long-lived credential to rotate, and a workflow author reaches a repository secret exactly as easily as `GITHUB_TOKEN`. Narrows nothing |
| Keep `environment: release` with a comment | Auto-creates an unprotected environment that looks like a gate to an audience the comment never reaches |
| Workflow-wide `contents: write` | Grants ref-write to six jobs that have no reason to write a ref |
| Fold the new disclosure into the existing `unenforced` entry | Loses the distinct fact that dispatching the lane *is* the release authority |

## From the change's testing.md

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

## Where these lessons go

- `specs/github/context.md`
