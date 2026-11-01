# Lesson bundle: prepare-specsync-6-1-2-package-metadata-and-release-handoff

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Prepare SpecSync 6.1.2 package metadata and release handoff
- **Kind**: Operations
- **Paths**: Cargo.toml, Cargo.lock, .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-validate-release-candidate.py, CHANGELOG.md, README.md, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, docs/releases/6.1.2.md, site/src/content/
- **Acceptance**: Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.1.2. The action.yml default, PUBLISHED_ACTION_DEFAULT and the Action docs default row stay published 6.1.1. README and site examples pin @v6.1.2 with version '6.1.2', except archive-gate examples, which keep @main until 6.1.2 is published.
- **Acceptance**: CHANGELOG.md has a 6.1.2 section listing every user-visible change merged since v6.1.1 (#817, #818, #821 including RUSTSEC-2026-0381 and RUSTSEC-2026-0382, #823, #824, #828, #829, #832), with #830 marked pending merge, and the Unreleased link compares from v6.1.2. docs/releases/6.1.2.md names the selected source, what release.yml does and does not do on tag, and every remaining gate.
- **Acceptance**: validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, the last including a control that publishing 6.1.2 ends the archive-gate @main allowance. cargo package --list --locked lists only the declared package files at 6.1.2.
- **Acceptance**: No runtime source, canonical spec, dependency, workflow logic, tag or release changes. Before the RC tag, #830 is either on main and merged into this branch or its CHANGELOG and handoff entries are removed.

## Evidence

- Verification commit: `7dc9b679a05b66d9c3d3202670628ec4ea5cb361`
- Base commit: `f240d379b9f023f5766e5c9b39508eb84e03e2bb`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

This is user-authorized release preparation only for spec-sync. The change remains draft. No definition approval or human implementation acceptance is claimed. Existing source branches and historical approval records are preserved. The release handoff names exact source and independent publication gates.

The branch was cut from main f240d379 and later updated by merging origin/main 709ab2bd (no rebase), which brought #823, #824, #832, #829 and #828 into the candidate. #830 (verification_ignored_paths) is listed as pending merge; it must be on main and merged here, or its entries removed, before the RC tag.

## From the change's design.md

# Design

Keep the runtime and public API byte-identical apart from the declared package version. Retain published installer defaults and existing supported publication destinations. A later release uses the reviewed and merged candidate.

## From the change's testing.md

# Testing

Run the repository native verification and release metadata checks. Record actual exits in the release handoff. Hosted platform qualification and publication readback remain pending. No green result will be inferred from historical receipts.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
