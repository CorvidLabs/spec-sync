---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: requirements
---

# Requirements

Extends `REQ-github-007`. No new requirement id: the release-qualification requirement already owns
tag authority, and splitting promotion identity into a second requirement would let one drift from
the other.

## R1 — Promotion uses the workflow's own token, and nothing else

`promote` SHALL create `refs/tags/vX.Y.Z` using `GITHUB_TOKEN`. `.github/**` SHALL contain no
reference to `actions/create-github-app-token`, `SPECSYNC_RELEASE_APP_ID`, or
`SPECSYNC_RELEASE_APP_PRIVATE_KEY`, in a step, an expression, or a comment. A half-removed App reads
as a policy that is temporarily off, and the next reader provisions a variable instead of learning
that promotion is now the workflow's own token.

## R2 — Write permission is job-scoped

The release workflow's top-level `permissions:` SHALL remain read-only (`contents: read`,
`actions: read`, `checks: read`). `contents: write` SHALL appear only on the jobs that must write —
`promote` (tag creation) and `release` (publication) — and `promote` SHALL declare no permission
beyond `contents: write`.

## R3 — No environment reference without an environment

`promote` SHALL NOT name a deployment environment while no protected `release` environment exists.
A referenced environment is auto-created without protection rules, so the reference would publish a
deployment gate that enforces nothing. The workflow SHALL record the order required to make it real:
create the environment with required reviewers and a `main`-only deployment branch policy, then
re-add the reference, then restore a check that proves those rules.

## R4 — The authority given up is stated where it is exercised

`UNENFORCED_TAG_POLICIES` SHALL include, in addition to the unrestricted-creation entry, that the
final tag is minted by the workflow's own `GITHUB_TOKEN` rather than a separate release identity,
and that promotion is not behind a deployment-environment gate. `release.yml` SHALL continue to fail
when that list is empty and SHALL continue to emit every entry as a `::warning::` annotation and
into the step summary on every run, green runs included.

The `promote` job SHALL additionally carry that statement in the workflow source, because a reader
auditing promotion opens the job, not a past run log.

## R5 — Nothing else weakens

Both immutability rulesets SHALL still be validated strictly: no bypass actor, no broadened
include/exclude, no extra or missing rule type, no non-`active` enforcement, no non-`Repository`
source. `promote` SHALL still require `resolve`, `validate`, and `authorize-release`. The empty-token
guard SHALL still refuse promotion before any tag is written.

## Acceptance

- `git grep -n "SPECSYNC_RELEASE_APP\|create-github-app-token" .github/` returns nothing.
- `actionlint` reports no issue in `release.yml` (shellcheck included).
- `python3 .github/scripts/test-validate-release-candidate.py` passes, including a promotion test
  that asserts the `GITHUB_TOKEN` shape, the absence of every App reference, the absence of any
  `environment:` key, and `contents: write` on `promote` alone.
- `rulesets` run against the live payloads of `21432132` and `21432148` exits 0 and emits three
  `unenforced` entries.
- `docs/ci-confidence.md` and `specs/github/**` describe `GITHUB_TOKEN`-minted promotion with no App
  and no environment gate.
