# Lesson bundle: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-1

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the Action default and current guidance to published SpecSync 6.4.1
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, action.yml, README.md, SECURITY.md, MIGRATION.md, CHANGELOG.md, docs/releases/6.4.1.md, docs/ADOPTING.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/index.md, specs/github/
- **Acceptance**: After the published v6.4.1 GitHub release (Latest, twelve assets; promoted from candidate v6.4.1-rc.1 at 3f0985e8, qualified on Ubuntu and macos-15 in run 44843175474), the main Action wrapper's omitted-input default moves to 6.4.1: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.4.1 and RC refs stay at 3f0985e8 with embedded default 6.4.0, and the docs say to pass version '6.4.1' with @v6.4.1; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary and manual-download text) names published 6.4.1; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.4.1.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. The PR merges only after crates.io serves 6.4.1; the floating v6 tag moves separately, after this wrapper commit's consumer checks.

## Evidence

- Verification commit: `5e1e017895a46cab73668dbaaf6d2b53c592e91c`
- Base commit: `3f0985e80acab0fe4559fe322cf929be40057414`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

- SpecSync 6.4.1 is published (2026-11-21): GitHub release `v6.4.1`, Latest, twelve assets, promoted from `v6.4.1-rc.1` at `3f0985e8` (release.yml runs 44843175474 qualify, 44851306915 dry run, 44854914914 promote); crates.io serves 6.4.1, and the Homebrew tap PR is CorvidLabs/homebrew-tap#44.
- As #1136 did for 6.4.0: the main Action wrapper's default and the current guidance move to the published release. The immutable `@v6.4.1` tag stays at `3f0985e8`, where `action.yml` defaults to `6.4.0`.
- Leif's answer, 2026-11-20 about 22:41Z, relayed by hc t4c: "Go (Recommended)" to releasing 6.4.1 with the same steps as 6.4.0, which named "then the Action default and @v6"; t4c confirmed it covers this change.

## From the change's design.md

# Design

- On exactly the lines #1136 moved for 6.4.0, the current release becomes 6.4.1 and the immutable tag's embedded default becomes 6.4.0: `action.yml`'s default and its description example, `PUBLISHED_ACTION_DEFAULT`, the Action docs' `version` row and published-binary text, README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md (install, CI pins, Trust `specsync-version`), the site index, the manual-download URL, and the github testing row.
- The two sentences the 6.4.1 preparation left reading "after publication" and "candidate" read as published, as #1136 did.
- Unchanged: historical text (the 6.2.0 floor, the relayed-decision requirement, MIGRATION.md's notes on 6.3.0 and 6.2.0, the comment-fallback note for `@v6.3.0` and later), the composite runtime scripts, the CI and Trust candidate mirrors, and every immutable tag.
- The github spec Purpose moves through one SpecSection delta. CHANGELOG `[Unreleased]` gains the entry, and `docs/releases/6.4.1.md` records publication.

## From the change's testing.md

# Testing

- `python3 -I .github/scripts/validate-release-version.py`, `validate-workflow-runtime-pins.py`, and `test-validate-release-candidate.py` (with Git's global configuration off for that one command on a tag-signing Mac, #1110).
- `fledge lanes run pre-push`; `specsync change audit --strict` with origin/main fetched.
- A scripted check that every line changed is one #1136 changed for 6.4.0, or one of its two sentences, and that the history lines are untouched.

## Where these lessons go

- `specs/github/github.3md`
