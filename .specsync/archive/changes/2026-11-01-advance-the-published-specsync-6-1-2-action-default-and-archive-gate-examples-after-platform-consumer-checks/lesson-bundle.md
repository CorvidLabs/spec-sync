# Lesson bundle: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the published SpecSync 6.1.2 Action default and archive-gate examples after platform consumer checks
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, action.yml, site/src/content/docs/integrations/github-action.md, README.md, CHANGELOG.md, docs/releases/6.1.2.md, specs/github/
- **Acceptance**: After the published v6.1.2 GitHub release (10 assets, Latest; candidate v6.1.2-rc.2 at d08d2746 qualified on Ubuntu and macos-15) and crates.io 6.1.2, advance the main Action wrapper's omitted-input default to 6.1.2: action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in site/src/content/docs/integrations/github-action.md. Composite runtime script bytes and the explicit CI and Trust candidate mirrors are unchanged.
- **Acceptance**: Archive-gate examples (require-archived, archive-gate-only) in README and site docs move from @main to @v6.1.2 with version '6.1.2'; the validator no longer allows @main pins for them, and README and CHANGELOG stop describing 6.1.2 as unpublished.
- **Acceptance**: Immutable v6.1.2 and RC refs stay at qualified candidate d08d2746 with embedded default 6.1.1; docs say to pass version '6.1.2' with @v6.1.2. The immutable v6.0.0 caveat is preserved. The github spec Purpose is updated through one normal SpecSection delta; permanent requirements and history rows are preserved.
- **Acceptance**: validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. Floating v6 moves only after this change is merged and omitted-input Linux and macOS Action consumers pass at the merged wrapper commit, with no immutable tag movement.

## Evidence

- Verification commit: `3fd288456f374879458702b78a47c71d703df69c`
- Base commit: `d08d2746f635bc26900601376d7392395d07faa9`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

SpecSync 6.1.2 is published on GitHub: release v6.1.2 (promotion run 41348603304) is Latest and not a prerelease, with the ten expected assets. Annotated v6.1.2 (tag object d16ef670) and v6.1.2-rc.2 (27e3c761) both identify candidate d08d2746, qualified on Ubuntu and macos-15 in run 41345066214. The macOS aarch64 archive matched its sidecar and prints `specsync 6.1.2`. crates.io publication is run by the owner; it is a precondition for approval of this change. Homebrew is homebrew-tap#40.

The immutable v6.1.2 tag embeds Action default 6.1.1 (PUBLISHED_ACTION_DEFAULT at release time). Archive-gate examples (`require-archived`, `archive-gate-only`) pin `@main` because `@v6.1.1` ignores those inputs; 6.1.2 is the first release that has them. This change mirrors #817 for 6.1.2. Floating v6 is currently tag object 189e2c19 at 621a7478 (default 6.1.1) and is not moved by this change.

## From the change's design.md

# Design

Advance only `PUBLISHED_ACTION_DEFAULT`, the `action.yml` version input default and description, the Action docs default row, and current README and CHANGELOG prose from 6.1.1 to 6.1.2. Move archive-gate examples from `@main` to `@v6.1.2` with `version: '6.1.2'`; with the published default at 6.1.2 the validator's archive-gate `@main` allowance ends by itself. The composite `runs` steps, downloader, checksum and runtime behavior, and the explicit CI and Trust candidate mirrors stay unchanged.

The github spec Purpose is updated through one MODIFIED SPEC SECTION Purpose delta; REQ-github-002 stays byte-identical. Immutable v6.1.2 keeps default 6.1.1, so docs tell `@v6.1.2` users to pass `version: '6.1.2'`. Floating v6 advances only after omitted-input Linux and macOS consumers pass at the merged wrapper commit.

## From the change's testing.md

# Testing

Run `validate-release-version.py` (expects published default 6.1.2 everywhere it checks and no archive-gate `@main` pins), `validate-workflow-runtime-pins.py`, `test-validate-release-candidate.py`, `change check --commit`, `change audit` and `fledge lanes run pre-push`, recording actual exits.

| Existing requirement | Evidence |
|----------------------|----------|
| REQ-github-002 (bytes unchanged) | Published v6.1.2 assets verified; validator agreement on the 6.1.2 default and docs; after merge, omitted-input Linux and macOS consumers at the merged wrapper SHA precede the floating v6 move. |

Native requirement_ids is expected to be empty for this Purpose-only delta. No result is inferred from historical receipts.

## Where these lessons go

- `specs/github/github.3md`
