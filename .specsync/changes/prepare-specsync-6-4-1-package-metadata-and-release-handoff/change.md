---
id: prepare-specsync-6-4-1-package-metadata-and-release-handoff
state: implementing
type: operations
base_commit: c14bdf690b33e2eda98de0fb1103298319b87bd9
---

# Prepare SpecSync 6.4.1 package metadata and release handoff

## Intent

Prepare SpecSync 6.4.1 package metadata and release handoff

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.4.1; README and site Action examples pin @v6.4.1 with version '6.4.1', and test-validate-release-candidate.py's fixtures for the version being released name 6.4.1, while the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row and current-install guidance stay at published 6.4.0; CHANGELOG.md has a dated 6.4.1 section listing every user-visible change merged since v6.4.0, with the Unreleased link comparing from v6.4.1; docs/releases/6.4.1.md names the selected source, the upgrade notes for each user-visible change since 6.4.0, what the release lane does, and the owner gates for the candidate tag, promotion, crates.io, the Homebrew tap, the post-publication PR and the floating v6 tag; validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.4.1; no tag, release or publish happens

## No-spec Rationale

Release package version, the self-built candidate inputs and release documentation only; no runtime, canonical spec, published Action default, risk or provenance policy changes
