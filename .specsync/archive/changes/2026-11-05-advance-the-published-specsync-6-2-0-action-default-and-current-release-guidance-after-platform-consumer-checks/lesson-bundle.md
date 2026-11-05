# Lesson bundle: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the published SpecSync 6.2.0 Action default and current release guidance after platform consumer checks
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, .github/scripts/test-validate-release-candidate.py, action.yml, README.md, SECURITY.md, MIGRATION.md, CHANGELOG.md, docs/releases/6.2.0.md, docs/ADOPTING.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/quickstart.md, site/src/content/docs/index.md, site/src/content/docs/cli.md, site/src/content/docs/workflow.md, site/src/content/examples/ci-gate.mdx, site/src/content/examples/polyglot.mdx, specs/github/
- **Acceptance**: After the published v6.2.0 GitHub release (Latest, ten assets; promoted from candidate v6.2.0-rc.2 at 40dcf33a, qualified on Ubuntu and macos-15 in run 42035620884) and crates.io 6.2.0, advance the main Action wrapper's omitted-input default to 6.2.0: action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in site/src/content/docs/integrations/github-action.md. Composite runtime script bytes and the explicit CI and Trust candidate mirrors are unchanged.
- **Acceptance**: Immutable v6.2.0 and RC refs stay at qualified candidate 40dcf33a with embedded default 6.1.2; docs say to pass version '6.2.0' with @v6.2.0. The immutable v6.0.0 caveat is preserved. The github spec Purpose is updated through one normal SpecSection delta; permanent requirements and history rows are preserved.
- **Acceptance**: Current guidance stops describing 6.2.0 as unpublished or pinning 6.1.2: README, SECURITY.md, MIGRATION.md current pins, docs/ADOPTING.md, the site index, quickstart, ci-gate and polyglot candidate notes, the github-action.md preparation paragraph and manual-CI download, and the cli and workflow reconcile-history availability wording. CHANGELOG dates 6.2.0 and gains an Unreleased entry; docs/releases/6.2.0.md records publication. Historical release text is unchanged.
- **Acceptance**: validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. Floating v6 moves only after this change is merged and omitted-input Linux and macOS Action consumers pass at the merged wrapper commit, with no immutable tag movement.

## Evidence

- Verification commit: `bf47f380099439a16be41e1cd4db5ca58b51f64c`
- Base commit: `40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

SpecSync 6.2.0 is published on GitHub: release v6.2.0 (promotion run 42043398852, dry run 42043257800) is Latest and not a prerelease, with the ten expected assets. Annotated v6.2.0 and v6.2.0-rc.2 both identify candidate 40dcf33a, qualified on Ubuntu and macos-15 in run 42035620884 (v6.2.0-rc.1 was cancelled at the old 45-minute macOS timeout). crates.io serves 6.2.0. Homebrew is CorvidLabs/homebrew-tap#41, open.

The immutable v6.2.0 tag embeds Action default 6.1.2 (PUBLISHED_ACTION_DEFAULT at release time). Archive-gate examples already pin `@v6.2.0`, and the `@main` allowance ended with 6.1.2, so no archive-gate edit is needed. This change mirrors #836 and its sibling guidance refresh for 6.2.0 in one change, so the release-validator test file is in scope from the start. Floating v6 is currently tag object 189e2c19 at 621a7478 (default 6.1.2) and is not moved by this change.

## From the change's design.md

# Design

Advance only `PUBLISHED_ACTION_DEFAULT`, the `action.yml` version input default and description, the Action docs default row, and current README, SECURITY, MIGRATION, ADOPTING, site and CHANGELOG prose from 6.1.2 to 6.2.0. Archive-gate examples already pin `@v6.2.0` with `version: '6.2.0'`. The composite `runs` steps, downloader, checksum and runtime behavior, and the explicit CI and Trust candidate mirrors stay unchanged. Validator logic is unchanged; the release-validator tests model the candidate window independently of the current default, so they need no fixture edit unless a run shows otherwise.

The github spec Purpose is updated through one MODIFIED SPEC SECTION Purpose delta; REQ-github-002 stays byte-identical. Immutable v6.2.0 keeps default 6.1.2, so docs tell `@v6.2.0` users to pass `version: '6.2.0'`. Floating v6 advances only after omitted-input Linux and macOS consumers pass at the merged wrapper commit.

## From the change's testing.md

# Testing

Run `validate-release-version.py` (expects published default 6.2.0 everywhere it checks), `validate-workflow-runtime-pins.py`, `test-validate-release-candidate.py`, `change check --commit`, `change audit` and `fledge lanes run pre-push`, recording actual exits.

| Existing requirement | Evidence |
|----------------------|----------|
| REQ-github-002 (bytes unchanged) | Published v6.2.0 assets verified; validator agreement on the 6.2.0 default and docs; after merge, omitted-input Linux and macOS consumers at the merged wrapper SHA precede the floating v6 move. |

Native requirement_ids is expected to be empty for this Purpose-only delta. No result is inferred from historical receipts.

## Where these lessons go

- `specs/github/github.3md`
