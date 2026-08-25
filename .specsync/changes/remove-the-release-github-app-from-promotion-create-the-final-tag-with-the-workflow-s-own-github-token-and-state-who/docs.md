---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: docs
---

# Documentation

A doc claiming an App-minted tag while `GITHUB_TOKEN` mints it is exactly the drift this project
exists to catch, so every surface that described the release App is updated in the same change that
removes it.

## `docs/ci-confidence.md` — "Tag authority: what is enforced, and what is not"

Was: two unenforced protections, with the App described as the *mechanism* promotion uses.

Now: **three** unenforced protections.

1. App-only final-tag creation is NOT enforced (unchanged, minus the sentence that said `promote`
   still mints an App token).
2. The final tag is minted by the workflow's own `GITHUB_TOKEN`, NOT by a separate release identity
   — bolded conclusion: *running the release lane and holding release authority are the same
   permission here.*
3. Promotion is NOT behind a deployment-environment gate.

Two paragraphs follow the list:

- **The environment decision**, stated as a decision with its reason: a referenced environment is
  auto-created without protection rules, so naming one this repository does not have would publish a
  gate that gates nothing. The route to a real gate is given in order — environment with rules
  first, reference second, proving check third.
- **What this does not cost**: no workflow here listens for a final-tag push, so the usual
  `GITHUB_TOKEN` objection has nothing to break; future work that must react to `vX.Y.Z` has to be
  called from inside `release.yml`.

The enforced half of the section — the two immutability rulesets, their patterns, and their empty
bypass lists — is untouched, because nothing about it changed.

## `specs/github/context.md`

The App paragraph now ends with a decision instead of a pending provisioning step, and gains the
`GITHUB_TOKEN` promotion shape, the environment removal with its reason, the per-run disclosure, and
the two facts that keep this bounded: immutability is untouched, and nothing triggers on a final tag.

## `specs/github/tasks.md`

The open task *"Decide the fate of App-only final-tag creation"* is checked off with the decision
that was actually made — **no GitHub App** — and what replaced it. A second, unchecked task records
the optional hardening path so that "no environment" stays a choice with a known remedy rather than
an omission someone rediscovers.

## In-code documentation

Two comment blocks carry the same statement to readers who never open the docs:

- `release.yml`, above `promote`: `WHO CAN MINT A RELEASE TAG`, `THE PROTECTION THAT WAS GIVEN UP`,
  `WHAT IS UNCHANGED`, and `NO \`environment:\` HERE, DELIBERATELY`. Pinned by a test, so deleting
  it fails the suite.
- `validate-release-candidate.py`, above `UNENFORCED_TAG_POLICIES`: why the list exists and why it
  may never be empty.

## Not updated, deliberately

`README.md`, `SECURITY.md`, `CONTRIBUTING.md`, and `site/` never described the release App or the
`release` environment — checked by grep, not assumed. Archived change packages under
`.specsync/archive/` still mention `SPECSYNC_RELEASE_APP_ID`; archives are history and are not
rewritten.
