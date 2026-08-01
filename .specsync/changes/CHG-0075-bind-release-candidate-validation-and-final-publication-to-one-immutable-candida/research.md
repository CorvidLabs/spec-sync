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

## Decision

Reuse the existing exact-SHA fail-closed pattern, but centralize RC identity and evidence validation
in one bounded script with tests. Use `release/vX.Y.Z` as the staging branch convention and annotated
`vX.Y.Z-rc.N` as the immutable marker. The final `vX.Y.Z` tag is a promotion output, not an input
used to discover whether the candidate works.

## Non-goals

- No broad workflow/YAML cleanup in this change; that receives a separate inventory-led package.
- No macOS/Windows execution on ordinary product PRs.
- No weakening of strict spec coverage, Trust identity, audit, or release artifact integrity.
