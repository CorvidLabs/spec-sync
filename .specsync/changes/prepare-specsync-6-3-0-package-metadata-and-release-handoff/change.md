---
id: prepare-specsync-6-3-0-package-metadata-and-release-handoff
state: draft
type: operations
base_commit: 084cf1ad81f4bcd05bdd675b19f4f00d4ce321f3
---

# Prepare SpecSync 6.3.0 package metadata and release handoff

## Intent

Prepare SpecSync 6.3.0 package metadata and release handoff

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.3.0; README and site Action examples pin @v6.3.0 with version '6.3.0' while the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row and current-install guidance stay published 6.2.0; CHANGELOG.md has a dated 6.3.0 section listing every user-visible change merged since v6.2.0 (including #1067's relay gate entry once it merges) with the Unreleased link comparing from v6.3.0; docs/releases/6.3.0.md names the selected source, the upgrade notes (once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later, while check-only uses keep working; relays on legacy workflow-v1 changes are not gated), what the release lane does, and every remaining owner gate; validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.3.0; no tag, release or publish happens

## No-spec Rationale

Release package version, the self-built candidate inputs and release documentation only; no runtime, canonical spec, published Action default, risk or provenance policy changes
