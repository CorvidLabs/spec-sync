# Lesson bundle: prepare-specsync-6-2-0-package-metadata-and-release-handoff

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Prepare SpecSync 6.2.0 package metadata and release handoff
- **Kind**: Operations
- **Paths**: Cargo.toml, Cargo.lock, .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-validate-release-candidate.py, CHANGELOG.md, README.md, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, docs/releases/6.2.0.md, site/src/content/
- **Acceptance**: Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.2.0. The action.yml default, PUBLISHED_ACTION_DEFAULT and the Action docs default row stay published 6.1.2. README and site examples, archive-gate examples included, pin @v6.2.0 with version '6.2.0' for use after publication, and current-install guidance (ADOPTING, MIGRATION, site index) names published 6.1.2.
- **Acceptance**: CHANGELOG.md has a 6.2.0 section listing every user-visible change merged since v6.1.2 (#836, #837, #838, #840, #841, #853), with #842, #845 and #846 included and marked pending merge in the handoff, and the Unreleased link compares from v6.2.0. docs/releases/6.2.0.md names the selected source, what release.yml does and does not do on tag, and every remaining gate.
- **Acceptance**: validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.2.0, and the pre-push gate passes.
- **Acceptance**: No runtime source, canonical spec, dependency, workflow logic, tag or release changes. Before the RC tag, #842, #845 and #846 are either on main and merged into this branch or their CHANGELOG and handoff entries are removed.

## Evidence

- Verification commit: `ae90be16cebf43fe3e09d2bf9850b1618c53ca03`
- Base commit: `61e8ca6a1c83eb0597149d6a415c6e1355089080`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

This is user-authorized release preparation only for spec-sync, delegated by the owner's session. 6.2.0 is a minor release because #846 adds the `change send-back` command. Existing source branches and historical approval records are preserved. The release handoff names exact source and independent publication gates.

The branch was cut from main 61e8ca6a, which already carries #836, #837, #838, #840, #841 and #853. #842, #845 and #846 were open when this was written; their CHANGELOG and handoff entries are included and marked pending merge. They must be on main and merged here (no rebase), or their entries removed, before the RC tag.

## From the change's design.md

# Design

Keep the runtime and public API byte-identical apart from the declared package version. Retain the published Action installer default (6.1.2) and existing supported publication destinations. Because the published default is already past 6.1.1, the archive-gate `@main` allowance has ended, so archive-gate examples pin the candidate like every other example. A later release uses the reviewed and merged candidate.

## From the change's testing.md

# Testing

Run the repository native verification, the three release metadata validators, the pre-push gate and `cargo test --bin specsync`. Record actual exits in the release handoff. Hosted platform qualification and publication readback remain pending. No green result will be inferred from historical receipts.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
