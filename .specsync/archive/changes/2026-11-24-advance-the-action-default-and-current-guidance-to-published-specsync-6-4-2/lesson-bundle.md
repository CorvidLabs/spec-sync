# Lesson bundle: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-2

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the Action default and current guidance to published SpecSync 6.4.2
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, action.yml, README.md, SECURITY.md, MIGRATION.md, CHANGELOG.md, docs/releases/6.4.2.md, docs/ADOPTING.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/index.md, specs/github/
- **Acceptance**: After the published v6.4.2 GitHub release (Latest, twelve assets; promoted from candidate v6.4.2-rc.2 at a275ab44, qualified on Ubuntu and macos-15 in run 45472594205), the main Action wrapper's omitted-input default moves to 6.4.2: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.4.2 and RC refs stay at a275ab44 with embedded default 6.4.1, and the docs say to pass version '6.4.2' with @v6.4.2; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary and manual-download text) names published 6.4.2; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.4.2.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. crates.io already serves 6.4.2; the floating v6 tag moves separately, after this wrapper commit's consumer checks.

## Evidence

- Verification commit: `6375f5242ff91da5d8ff0152e5cfe4ff95d231e5`
- Base commit: `a275ab44daffac2d562df9a6fc05dc6a9a559059`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

- SpecSync 6.4.2 is published (2026-11-24): GitHub release `v6.4.2`, Latest, twelve assets, promoted from `v6.4.2-rc.2` at `a275ab44` (release.yml runs 45472594205 qualify, 45489088237 dry run, 45489261983 promote); crates.io serves 6.4.2, and the Homebrew tap carries it (CorvidLabs/homebrew-tap#45, merged).
- As #1142 did for 6.4.1: the main Action wrapper's default and the current guidance move to the published release. The immutable `@v6.4.2` tag stays at `a275ab44`, where `action.yml` defaults to `6.4.1`.
- The 6.4.2 preparation (#1156) already moved the YAML examples to `@v6.4.2` with `version: '6.4.2'` and left three sentences reading "after publication"; they read as published here.
- Leif asked for the cargo and Homebrew publication steps in the release-lead chat on 2026-11-24 and is asked there to approve this scope; his words are in the approval record.

## From the change's design.md

# Design

- On exactly the lines #1142 moved for 6.4.1, the current release becomes 6.4.2 and the immutable tag's embedded default becomes 6.4.1: `action.yml`'s default and its description, `PUBLISHED_ACTION_DEFAULT`, the Action docs' `version` row and published-binary text, README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md (install, CI pins, Trust `specsync-version`), the site index, the manual-download URL, and the github testing row.
- The three sentences the 6.4.2 preparation left reading "after publication" (one in README, two in the Action docs) read as published, in the words #1142 used.
- Unchanged: historical text (the 6.2.0 floor, the relayed-decision requirement, MIGRATION.md's notes on earlier releases, the comment-fallback note for `@v6.3.0` and later), the composite runtime scripts, the CI and Trust candidate mirrors, and every immutable tag.
- The github spec Purpose moves through one SpecSection delta. CHANGELOG `[Unreleased]` gains the entry, and `docs/releases/6.4.2.md` records publication.

## From the change's testing.md

# Testing

- `python3 -I .github/scripts/validate-release-version.py`, `validate-workflow-runtime-pins.py`, and `test-validate-release-candidate.py`.
- `fledge lanes run pre-push`; `specsync change audit --strict` with origin/main fetched.
- A scripted check that every line changed is one #1142 changed for 6.4.1, one of the three "after publication" sentences, the CHANGELOG entry or the release handoff, and that the history lines are untouched.
- The pull request's own Action checks run this wrapper commit with its new default on the hosted runners.

## Where these lessons go

- `specs/github/github.3md`
