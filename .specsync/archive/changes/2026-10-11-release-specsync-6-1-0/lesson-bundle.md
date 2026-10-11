# Lesson bundle — release-specsync-6-1-0

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Release SpecSync 6.1.0
- **Kind**: Documentation
- **Specs**: github
- **Paths**: Cargo.toml, Cargo.lock, action.yml, .github/workflows/ci.yml, .github/workflows/trust.yml, site/, specs/github/, README.md, SECURITY.md, CHANGELOG.md, MIGRATION.md, docs/ADOPTING.md
- **Acceptance**: Cargo.toml, Cargo.lock, the Action default, CI and Trust pins, README, the site, and SECURITY name 6.1.0. CHANGELOG has a 6.1.0 section and the Unreleased link starts at v6.1.0. The three release validators pass. No tag is pushed.

## Evidence

- Verification commit: `7bf70fff9be44983ec1775814845fcfd838d33c6`
- Base commit: `efe441d0ee9e73a687dbd459d4f7f26b0c68f7d7`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

Published SpecSync is 6.0.1. Main at efe441d has seven commits after that tag: optional module 3md files, lesson briefings, `change pr`, archive readiness, a fail-closed broken policy, finalize committing the archive move, the source split, and a handoff that does not claim a commit. Corvid-agent recommended 6.1.0 because the 3md support is new behavior a 6.0.1 user can use. The source split is not the reason for the minor.

This change only prepares that version. It does not push a tag, promote a candidate, or publish crates.io or Homebrew. Floating `v6` stays on the 6.0.1 commit until a person moves it after promotion. The immutable `@v6.0.0` tag stays on `6.0.0-rc.14`. Pull request #800 stays out of this release.

## From the change's design.md

# Design

No interface or layout change. The version strings a person copies from the README and the site are the only surface.

## From the change's testing.md

# Testing

REQ-github-002 is checked by `python3 -S .github/scripts/validate-release-version.py`, which requires the Action default, the consumer pins, the README and site pins, the SECURITY major ref, and the changelog link to match Cargo.toml. Also run `python3 -S .github/scripts/validate-workflow-runtime-pins.py` and `python3 .github/scripts/test-validate-release-candidate.py`.

## Where these lessons go

- `specs/github/github.3md`
