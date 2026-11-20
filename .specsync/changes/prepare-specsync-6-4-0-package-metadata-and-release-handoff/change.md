---
id: prepare-specsync-6-4-0-package-metadata-and-release-handoff
state: implementing
type: operations
base_commit: 10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f
---

# Prepare SpecSync 6.4.0 package metadata and release handoff

## Intent

Prepare SpecSync 6.4.0 package metadata and release handoff

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.4.0; README and site Action examples pin @v6.4.0 with version '6.4.0', and test-validate-release-candidate.py's fixtures for the version being released name 6.4.0, while the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row and current-install guidance stay at published 6.3.0; CHANGELOG.md has a dated 6.4.0 section listing every user-visible change merged since v6.3.0, with the Unreleased link comparing from v6.4.0; docs/releases/6.4.0.md names the selected source, the upgrade notes (what released 6.3.0 does in a repository where this version has written history fragments, stated from a run of both binaries; change ship-status exits 1 when it reports a blocker; change approve --dry-run, its --json shape, and that a fix may hold a placeholder and is not safe to run as printed), what the release lane does, and the owner gates for the candidate tag, promotion, crates.io, the Homebrew tap, the post-publication PR and the floating v6 tag; validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.4.0; no tag, release or publish happens

## No-spec Rationale

Release package version, the self-built candidate inputs and release documentation only; no runtime, canonical spec, published Action default, risk or provenance policy changes
