---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: context
---

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
