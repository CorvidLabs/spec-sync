# Lesson bundle: prepare-specsync-6-4-1-package-metadata-and-release-handoff

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Prepare SpecSync 6.4.1 package metadata and release handoff
- **Kind**: Operations
- **Paths**: Cargo.toml, Cargo.lock, .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-validate-release-candidate.py, CHANGELOG.md, README.md, docs/releases/6.4.1.md, site/src/content/
- **Acceptance**: Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.4.1; README and site Action examples pin @v6.4.1 with version '6.4.1', and test-validate-release-candidate.py's fixtures for the version being released name 6.4.1, while the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row and current-install guidance stay at published 6.4.0; CHANGELOG.md has a dated 6.4.1 section listing every user-visible change merged since v6.4.0, with the Unreleased link comparing from v6.4.1; docs/releases/6.4.1.md names the selected source, the upgrade notes for each user-visible change since 6.4.0, what the release lane does, and the owner gates for the candidate tag, promotion, crates.io, the Homebrew tap, the post-publication PR and the floating v6 tag; validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.4.1; no tag, release or publish happens

## Evidence

- Verification commit: `102022428a287fa154d7c717325b03e7d7382e5e`
- Base commit: `c14bdf690b33e2eda98de0fb1103298319b87bd9`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

- Leif's go for SpecSync 6.4.1 with the same steps as 6.4.0 ("Go (Recommended)", 2026-11-20 about 22:41Z), and his approval of this definition, with "1: hardening only (Recommended)" for what 6.4.1 holds (about 22:44Z), both relayed by hc t4c. Only this prep PR and the 6.4.1 fix merge before the `v6.4.1` tag; the reviewed 6.5.0 PRs merge after it.
- 6.4.0's prep was #1109; this is the same change with the versions moved. The 6.4.1 fix opens as its own PR and adds its own lines under `## [6.4.1]`.

## From the change's design.md

# Design

`validate-release-version.py` decides which surfaces move. At Cargo version 6.4.1 on main `c14bdf69` it asks for exactly these:

- **Moving to 6.4.1:** `Cargo.toml` and the `specsync` entry in `Cargo.lock`; `ci.yml`'s packaged Action consumer `version` and `trust.yml`'s `specsync-version`; every documented Action step in `README.md` and `site/src/content` (`@v6.4.1` with `version: '6.4.1'`); the CHANGELOG heading and the `[Unreleased]` and `[6.4.1]` links; the anchors for the version being released in `test-validate-release-candidate.py`.
- **Staying at published 6.4.0:** the `action.yml` default, `PUBLISHED_ACTION_DEFAULT`, the docs default row and current-install guidance. Prose about the immutable `@v6.4.0` tag keeps `version: '6.4.0'`.
- **The CHANGELOG** moves the `[Unreleased]` entries under `## [6.4.1] - 2026-11-21`, the planned tag day, corrected before the tag if the day moves.
- **`docs/releases/6.4.1.md`** has the sections of `docs/releases/6.4.0.md`. Its "Selected source" says the candidate is cut from `origin/main` at the commit Leif names with his go for the candidate tag.

## From the change's testing.md

# Testing

- `python3 -I .github/scripts/validate-release-version.py` reports 6.4.1; `validate-workflow-runtime-pins.py` and `test-validate-release-candidate.py` pass (the last with Git's global configuration off for that one command on a tag-signing Mac, #1110).
- `fledge lanes run pre-push` and `fledge lanes run verify`; `specsync change audit --strict` with origin/main fetched.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
