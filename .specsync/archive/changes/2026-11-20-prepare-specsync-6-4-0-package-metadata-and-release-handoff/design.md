---
change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
artifact: design
---

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
