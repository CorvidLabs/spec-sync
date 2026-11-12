# Lesson bundle: prepare-specsync-6-3-0-package-metadata-and-release-handoff

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Prepare SpecSync 6.3.0 package metadata and release handoff
- **Kind**: Operations
- **Paths**: Cargo.toml, Cargo.lock, .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-validate-release-candidate.py, CHANGELOG.md, README.md, docs/releases/6.3.0.md, site/src/content/
- **Acceptance**: Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.3.0; README and site Action examples pin @v6.3.0 with version '6.3.0' while the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row and current-install guidance stay published 6.2.0; CHANGELOG.md has a dated 6.3.0 section listing every user-visible change merged since v6.2.0 (including #1067's relay gate entry once it merges) with the Unreleased link comparing from v6.3.0; docs/releases/6.3.0.md names the selected source, the upgrade notes (once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later, while check-only uses keep working; relays on legacy workflow-v1 changes are not gated), what the release lane does, and every remaining owner gate; validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.3.0; no tag, release or publish happens

## Evidence

- Verification commit: `0da3ba8bd3d01bafdfac28e806e39716c0b275bb`
- Base commit: `084cf1ad81f4bcd05bdd675b19f4f00d4ce321f3`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Leif decided to release (coordinator chat, 2026-11-12, relayed by hc t370): "yes chough 0.6.0 ships with spec-sync 6.3.0". The coordinator assigned this preparation as task 2 and marked it "Do not merge until Leif says go". No tag, release, publish or workflow dispatch happens without his explicit go.

- **6.3.0, not 6.2.1:** main has added commands since `v6.2.0` (doctor, retire, repair-relays, relayed decisions, `--timeout`), and a new download (aarch64 musl).
- **Starting points:** claude-code-t37j's 6.2.1 draft (`_audits/spec-sync-6.2.1-draft-2026-11-11.md`), reworked to 6.3.0, with everything merged since included. The 6.2.0 precedent is #859, release prep, and #867, the post-release Action default.
- **#1067**, the relay gate (#1064), changes the relayed-decision CHANGELOG entry and adds the upgrade note. It must merge to main, and main must be merged into this branch, before the RC tag. That merge is the one update this preparation still waits for. The note: "once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later; check-only uses keep working".

## From the change's design.md

# Design

`validate-release-version.py` decides which surfaces move.

- **Moving to 6.3.0:**
  - `Cargo.toml` and `Cargo.lock`;
  - the self-built candidate inputs (`ci.yml`'s packaged Action consumer `version`, `trust.yml`'s `specsync-version`);
  - every documented Action step in `README.md` and `site/src/content` (`@v6.3.0` with `version: '6.3.0'`);
  - the CHANGELOG heading and links;
  - the candidate anchors in `test-validate-release-candidate.py`, as #859 moved them for 6.2.0.
- **Staying at published 6.2.0:** the `action.yml` default, `PUBLISHED_ACTION_DEFAULT`, the docs default row and current-install guidance. Prose about the immutable `@v6.2.0` tag keeps `version: '6.2.0'`.
- **The CHANGELOG** keeps every existing `[Unreleased]` entry, under a dated `[6.3.0]` heading. It adds the user-visible entries main lacked, and a Security section for RUSTSEC-2026-0482. Internal CI and test-harness changes stay out, as in 6.2.0.

## From the change's testing.md

# Testing

| Check | Expected |
|---|---|
| `python3 .github/scripts/validate-release-version.py` | "Validated release version 6.3.0 across current distribution surfaces" |
| `python3 .github/scripts/validate-workflow-runtime-pins.py` | passes |
| `python3 .github/scripts/test-validate-release-candidate.py` | OK |
| `fledge lanes run pre-push` | pass |
| Review of the diff | the `action.yml` default, docs default row, `PUBLISHED_ACTION_DEFAULT` and current-install guidance unchanged; prose about the immutable `@v6.2.0` tag unchanged |

Hosted CI on the archived head runs the full lane. The release workflows run only on Leif's RC tag and dispatch.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
