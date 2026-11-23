# Lesson bundle: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-0

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the Action default and current guidance to published SpecSync 6.4.0
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, action.yml, README.md, SECURITY.md, MIGRATION.md, CHANGELOG.md, docs/releases/6.4.0.md, docs/ADOPTING.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/index.md, specs/github/
- **Acceptance**: After the published v6.4.0 GitHub release (Latest, twelve assets; promoted from candidate v6.4.0-rc.1 at 3d57fa13, qualified on Ubuntu and macos-15 in run 44781969984), the main Action wrapper's omitted-input default moves to 6.4.0: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.4.0 and RC refs stay at 3d57fa13 with embedded default 6.3.0, and the docs say to pass version '6.4.0' with @v6.4.0; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary and manual-download text) names published 6.4.0; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.4.0.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. The PR merges only after crates.io serves 6.4.0; the floating v6 tag moves separately, after this wrapper commit's consumer checks.

## Evidence

- Verification commit: `74ff023537083769c200a7d5346f23d2d9c18bc1`
- Base commit: `3d57fa13f298a46e52416ab0407cb57d9dc67828`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

- SpecSync 6.4.0 is published (2026-11-20): GitHub release `v6.4.0`, Latest, twelve assets, promoted from `v6.4.0-rc.1` at `3d57fa13` (release.yml runs 44781969984 qualify, 44794356291 dry run, 44801524176 promote); crates.io serves 6.4.0.
- Release plan step 12, as #1069 did for 6.3.0 and #867 for 6.2.0: the main Action wrapper's default and the current guidance move to the published release. The immutable `@v6.4.0` tag stays at `3d57fa13`, where `action.yml` defaults to `6.3.0`.
- Leif's answer, 2026-11-20 about 19:21Z, relayed by hc t4c: "Yes, both (Recommended)" to the post-publication PR and the floating `v6` tag move.

## From the change's design.md

# Design

- On exactly the lines #1069 moved for 6.3.0, the current release becomes 6.4.0 and the immutable tag's embedded default becomes 6.3.0: `action.yml`'s default and its description example, `PUBLISHED_ACTION_DEFAULT`, the Action docs' `version` row and published-binary text, README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md (install, CI pins, Trust `specsync-version`), the site index, the manual-download URL, and the github testing row.
- The two sentences the 6.4.0 preparation left saying "after publication" or "candidate" now read as published, as #1069 did.
- Unchanged: historical text (the 6.2.0 floor, the relayed-decision requirement, the comment-fallback note for `@v6.3.0` and later), the composite runtime scripts, the CI and Trust candidate mirrors, and every immutable tag.
- The github spec Purpose moves through one SpecSection delta. CHANGELOG `[Unreleased]` gains the entry, and `docs/releases/6.4.0.md` records publication.

## From the change's testing.md

# Testing

- `python3 -I .github/scripts/validate-release-version.py`, `validate-workflow-runtime-pins.py`, and `test-validate-release-candidate.py` (with Git's global configuration off for that one command on a tag-signing Mac, #1110).
- `fledge lanes run pre-push` and `fledge lanes run verify`; `specsync change audit --strict` with origin/main fetched.
- A scripted check that every line changed is one #1069 changed for 6.3.0, or one of its two sentences, and that the history lines are untouched.

## Where these lessons go

- `specs/github/github.3md`
