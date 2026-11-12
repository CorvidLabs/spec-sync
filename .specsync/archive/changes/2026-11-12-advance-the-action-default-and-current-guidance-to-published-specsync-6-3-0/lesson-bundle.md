# Lesson bundle: advance-the-action-default-and-current-guidance-to-published-specsync-6-3-0

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Advance the Action default and current guidance to published SpecSync 6.3.0
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-version.py, action.yml, README.md, SECURITY.md, MIGRATION.md, CHANGELOG.md, docs/releases/6.3.0.md, docs/ADOPTING.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/index.md, specs/github/
- **Acceptance**: After the published v6.3.0 GitHub release (Latest, twelve assets; promoted from candidate v6.3.0-rc.1 at b87bbd88, qualified on Ubuntu and macos-15 in run 43162493255), the main Action wrapper's omitted-input default moves to 6.3.0: the action.yml version default and description, PUBLISHED_ACTION_DEFAULT in validate-release-version.py, and the version default row in the Action docs; composite runtime script bytes and the CI and Trust candidate mirrors are unchanged. Immutable v6.3.0 and RC refs stay at b87bbd88 with embedded default 6.2.0, and the docs say to pass version '6.3.0' with @v6.3.0; the immutable v6.0.0 caveat is kept. The github spec Purpose changes through one SpecSection delta. Current guidance (README install and pin text, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, the site index, the Action docs' published-binary, comment-fallback and manual-download text) names published 6.3.0; historical release text is unchanged. CHANGELOG gains an Unreleased entry and docs/releases/6.3.0.md records publication. validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, and the pre-push gate passes. The PR merges only after crates.io serves 6.3.0; floating v6 moves only on Leif's separate decision.

## Evidence

- Verification commit: `07dbc27bbb468f1e839dad03087d85014070c283`
- Base commit: `b87bbd88efea06ffd9474af02018240f24300419`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

SpecSync 6.3.0 is published.

- **The release:** GitHub release `v6.3.0` is Latest with twelve assets (promotion run 43168578964). It was promoted from `v6.3.0-rc.1` at `b87bbd88`, which qualified on Ubuntu and macos-15 (run 43162493255).
- **Authority:** Leif's go relayed by hc t370 (2026-11-12). t370 asked for this post-publication defaults change, as #867 was for 6.2.0.
- **Timing:** `cargo publish` is on hold for Leif's separate crates.io go, and the guidance here says crates.io serves 6.3.0. So this PR merges only after that publish.
- **Elsewhere:** the Homebrew bump is CorvidLabs/homebrew-tap#42. Moving floating `v6` is a separate decision for Leif.

## From the change's design.md

# Design

This follows #867 exactly, one version later.

- **Moving to 6.3.0:**
  - the `action.yml` `version` default and its example;
  - `PUBLISHED_ACTION_DEFAULT`;
  - the docs `| version |` row;
  - current guidance in README, SECURITY.md, MIGRATION.md, ADOPTING.md, the site index and the Action docs.
- **The tag notes:** immutable `@v6.3.0` embeds default `6.2.0` (`action.yml` at `b87bbd88`), so the docs say to pass `version: '6.3.0'` with it.
- **The #886 comment fallback** is in `@v6.3.0`, and not in `@v6.2.0` or earlier. The Action docs now say so.
- **Unchanged:** historical lines (the glibc floor up to 6.2.0, `reconcile-history` available from 6.2.0, archive-gate and `@v6.1.2` notes), the Action shell scripts, and the CI and Trust candidate mirrors.
- **The github spec Purpose** changes through one SpecSection delta.

## From the change's testing.md

# Testing

| Check | Expected |
|---|---|
| `validate-release-version.py` | "Validated release version 6.3.0 across current distribution surfaces", with the published default equal to the Cargo version |
| `test-validate-release-candidate.py` | OK |
| `validate-workflow-runtime-pins.py` | passes |
| `fledge lanes run pre-push` | pass |
| `git show v6.3.0:action.yml` | default `6.2.0` and the #886 403 fallback present, matching the docs |

The PR's CI runs the packaged Action consumer and the full lane. Omitted-input Action consumers on Linux and macOS at the merged wrapper commit come before any floating `v6` move, which is Leif's decision.

## Where these lessons go

- `specs/github/github.3md`
