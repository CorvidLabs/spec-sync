# Lesson bundle: prepare-specsync-6-4-2-package-metadata-and-release-handoff

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Prepare SpecSync 6.4.2 package metadata and release handoff
- **Kind**: Operations
- **Paths**: Cargo.toml, Cargo.lock, .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-validate-release-candidate.py, CHANGELOG.md, README.md, docs/releases/6.4.2.md, site/src/content/
- **Acceptance**: Cargo metadata and the self-built CI and Trust pins select 6.4.2. Action examples name the explicit candidate version. The installed binary default and current-install guidance select published 6.4.1 until publication. The changelog and release handoff account for user-visible fixes actually merged since v6.4.1. The version validator, workflow-pin validator, release-candidate contract tests and project gates pass on the prepared revision.

## Evidence

- Verification commit: `7fb04e62b05b2355db13b2bd54568a9ef6be7982`
- Base commit: `71f1867d7ddcf2b075b49c421d2f87ea0b760be1`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

The latest GitHub release is v6.4.1. The maintainer requested a bug-fix release and the integration of existing correctness PRs. The release comparison starts at v6.4.1. Record fixes under the new version once they reach main. The three fixes already on main synchronize change state, surface frozen Change Log deltas during audit, and update the url lockfile entry.

At preparation time, main still resolves cookie 0.18.1. PR #1148 addresses the current advisory. A release candidate needs the dependency audit and the hosted checks to pass on the selected main revision.

Final reconcile, 2026-11-24. The release contents are the 21 pull requests merged since v6.4.1, through #1149. Main's nine new Unreleased entries moved into the 6.4.2 section, and #1105 gained the entry #1149 did not add, so the section holds 32 entries and Unreleased is empty. The handoff has one line per pull request. Cargo.lock differs from main only in the package version, and the hosted dependency audit passes on main at `6c5dc5b2`.

One thing dogfooding showed and this change leaves alone: `specsync stale` reports `cmd_change` six commits behind its source on main, where it reported nothing at v6.4.1. No gate runs that command.

## From the change's design.md

# Design

Use the release-preparation surfaces established by #1138. Cargo, the CI action consumer, the Trust workflow, the candidate test fixtures and the documented explicit Action examples reference 6.4.2. The release validator uses 6.4.1 for published defaults until the release is available.

The handoff identifies the source revision and release evidence as they become available. Link each user-visible fix to its merged PR. The documented release workflow qualifies the immutable candidate on Linux and macOS before promotion.

## From the change's testing.md

# Testing

Run the distribution version validator, workflow runtime pin validator and release-candidate contract tests. Check both the candidate inputs and the published default in an isolated checkout. Confirm the release notes describe the diff from v6.4.1 to the selected source.

Run the full verification lane, strict check with forced 100 percent coverage, scoped verification, strict change audit, the pre-push gate and configured Trust. Record the exact commit for each result. The candidate workflow supplies the Linux and macOS qualification evidence after tagging.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
