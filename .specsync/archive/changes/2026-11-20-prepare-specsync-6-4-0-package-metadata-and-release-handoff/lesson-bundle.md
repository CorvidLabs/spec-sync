# Lesson bundle: prepare-specsync-6-4-0-package-metadata-and-release-handoff

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Prepare SpecSync 6.4.0 package metadata and release handoff
- **Kind**: Operations
- **Paths**: Cargo.toml, Cargo.lock, .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-validate-release-candidate.py, CHANGELOG.md, README.md, docs/releases/6.4.0.md, site/src/content/
- **Acceptance**: Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.4.0; README and site Action examples pin @v6.4.0 with version '6.4.0', and test-validate-release-candidate.py's fixtures for the version being released name 6.4.0, while the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row and current-install guidance stay at published 6.3.0; CHANGELOG.md has a dated 6.4.0 section listing every user-visible change merged since v6.3.0, with the Unreleased link comparing from v6.4.0; docs/releases/6.4.0.md names the selected source, the upgrade notes (what released 6.3.0 does in a repository where this version has written history fragments, stated from a run of both binaries; change ship-status exits 1 when it reports a blocker; change approve --dry-run, its --json shape, and that a fix may hold a placeholder and is not safe to run as printed), what the release lane does, and the owner gates for the candidate tag, promotion, crates.io, the Homebrew tap, the post-publication PR and the floating v6 tag; validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.4.0; no tag, release or publish happens

## Evidence

- Verification commit: `a7d5c658de4f3008a55fb5e29bc28267d4314934`
- Base commit: `10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Leif set what 6.4.0 holds and the order of its steps (Claude Code chat with his planning session hc t4c, 2026-11-19 about 15:49Z and 15:53Z, relayed through ss-lead hc t58). ss-lead assigned this preparation on 2026-11-20. Main is held until the `v6.4.0` tag. Leif approves this definition, reviews the PR and says yes to its merge. No tag, release, publish or workflow dispatch happens here.

- **6.4.0, not 6.3.1:** since `v6.3.0`, main adds a flag (`change approve --dry-run`, #1100) and changes an exit code (`change ship-status` exits 1 on a blocker, #1093). #1086 and #1088 change what `change check` writes: a history fragment, where it used to add a Change Log row and bump the version.
- **Merged since `v6.3.0`:** #1069 (the Action default to published 6.3.0), #1073 and #1078 (this repository's own Chough records), #1086 and #1088 (history fragments), #1093 (#654, #1038, #937) and #1100 (#1091).
- **Precedent:** #1068 prepared 6.3.0 and #1069 moved the published defaults after it. This change follows #1068.

## From the change's design.md

# Design

`validate-release-version.py` decides which surfaces move. At Cargo version 6.4.0 on main 10d9bb3f, it asks for exactly these:

- **Moving to 6.4.0:**
  - `Cargo.toml` and the `specsync` entry in `Cargo.lock`;
  - the self-built candidate inputs (`ci.yml`'s packaged Action consumer `version`, `trust.yml`'s `specsync-version`);
  - every documented Action step in `README.md` and `site/src/content` (`@v6.4.0` with `version: '6.4.0'`);
  - the CHANGELOG heading and the `[Unreleased]` and `[6.4.0]` links;
  - the anchors for the version being released in `test-validate-release-candidate.py`, as #1068 moved them for 6.3.0.
- **Staying at published 6.3.0:** the `action.yml` default, `PUBLISHED_ACTION_DEFAULT`, the docs default row and current-install guidance. Prose about the immutable `@v6.3.0` tag keeps `version: '6.3.0'`.
- **The CHANGELOG** keeps every existing `[Unreleased]` entry, under `## [6.4.0] - 2026-11-24`, the planned tag date, and adds any user-visible change main lacks. The date is corrected before the tag if the day moves. Internal CI, test-harness and repository-record changes stay out, as in 6.3.0.
- **`docs/releases/6.4.0.md`** has the sections of `docs/releases/6.3.0.md`. Its "Selected source" says the candidate is cut from `origin/main` at the commit Leif names when he gives the go for the candidate tag, and that no other SpecSync change may be open before the tag. It does not say the candidate is cut from this PR's merge. Each upgrade note is stated from a run, never from memory. What released 6.3.0 does in a repository where 6.4.0 wrote history fragments comes from running both binaries on one fixture.

## From the change's testing.md

# Testing

| Check | Expected |
|---|---|
| `python3 .github/scripts/validate-release-version.py` | "Validated release version 6.4.0 across current distribution surfaces" |
| `python3 .github/scripts/validate-workflow-runtime-pins.py` | passes |
| `python3 .github/scripts/test-validate-release-candidate.py` | OK |
| Released 6.3.0 and this version on one fixture where this version shipped a change | 6.3.0's `check`, `change audit`, `change audit --strict` and `change check` behave as the upgrade note says |
| `fledge lanes run pre-push` | pass |
| `specsync change audit --strict` | pass |
| Review of the diff | the `action.yml` default, docs default row, `PUBLISHED_ACTION_DEFAULT` and current-install guidance unchanged; prose about the immutable `@v6.3.0` tag unchanged |

Hosted CI on the archived head runs the full lane. The release workflows run only on Leif's RC tag and dispatch.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
