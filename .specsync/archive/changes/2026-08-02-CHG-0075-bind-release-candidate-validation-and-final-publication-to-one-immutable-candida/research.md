---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: research
---

# Research

## Current topology

- `.github/workflows/ci.yml` contains the ordinary `ubuntu-latest`, `macos-latest`, and
  `windows-latest` test matrix.
- `.github/workflows/release.yml` triggers on `v*`, validates the release commit, builds platform
  artifacts, and uploads a GitHub release after the final tag exists.
- `.github/workflows/lifecycle-policy-guard.yml` demonstrates fail-closed exact candidate-SHA
  fetching for protected policy checks.
- `fledge.toml` has full local and CI lanes but no release-candidate platform lane.
- `post-merge-archive.yml` emits archive binding from `pull_request`, while release validation
  currently asks the resolved workflow run for `pull_request_target`; the two surfaces must share
  one exact event/path/SHA contract.

## Decision

Reuse the existing exact-SHA fail-closed pattern, but centralize RC identity and evidence validation
in one bounded script with tests. Use `release/vX.Y.Z` as the staging branch convention and annotated
`vX.Y.Z-rc.N` as the immutable marker. The final `vX.Y.Z` tag is a promotion output, not an input
used to discover whether the candidate works.

## Prevention lesson

Trigger: a visible green check can outlive or be detached from the bytes it originally qualified.

Root cause: tag names, check names, and downloaded artifacts are labels unless each is resolved and
cross-checked against one full candidate SHA at the point of use.

Invariant: authorization re-resolves the annotated RC object, rejects conflicting run history,
authenticates the exact release-workflow check, downloads the original per-platform records, and
validates their common tag/SHA/lane/workflow revision. Publication independently verifies the final
tag, checkout, artifact provenance manifest, and archive checksum against that candidate.

Regression coverage: deterministic validator fixtures cover malformed/lightweight/moved markers,
missing/duplicate/failed/mixed evidence, stale workflow revisions, final-tag mismatches, and
tampered or mixed-SHA artifact manifests; workflow assertions cover Ubuntu-only development and
the real `pull_request` archive-binding event.

Review-triggered extension: creation ordering must also be enforced outside YAML. One active policy
allows a human to create an RC marker but forbids every update/deletion; a second permits only a
dedicated CorvidLabs release GitHub App to create final semver tags; a third forbids every actor,
including that App, from updating or deleting final tags. The workflow validates all three remote
policies before qualification or promotion. The App private key is an
environment secret available only to the protected `release` promotion job; that job mints one
short-lived installation token restricted to the repository and disables checkout credentials.
Every Action in the release trust chain is
pinned to a full commit SHA, and downloaded executables are checked against digests committed in the
workflow rather than checksums fetched from the same release. Publication re-resolves both tags and
actual `HEAD` after builds, then re-downloads the original platform evidence and re-hashes every
release archive. Artifact uploads use overwrite-safe names so a failed-job rerun cannot be trapped
by stale evidence.

Sandbox finding: GitHub rejected the built-in GitHub Actions integration as a repository-ruleset
bypass because that integration is not owned by the repository or organization, then rejected the
deploy-key fallback because CorvidLabs disables deploy keys. The supported boundary is therefore a
dedicated organization-installed GitHub App. The regression contract binds the final-tag bypass to
the configured App id, rejects every RC bypass and broader actor, and requires the promotion job to
mint its repository-scoped token only inside the protected environment.

## Non-goals

- No broad workflow/YAML cleanup in this change; that receives a separate inventory-led package.
- No macOS/Windows execution on ordinary product PRs.
- No weakening of strict spec coverage, Trust identity, audit, or release artifact integrity.
