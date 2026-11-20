---
change: prepare-specsync-6-4-1-package-metadata-and-release-handoff
artifact: design
---

# Design

`validate-release-version.py` decides which surfaces move. At Cargo version 6.4.1 on main `c14bdf69` it asks for exactly these:

- **Moving to 6.4.1:** `Cargo.toml` and the `specsync` entry in `Cargo.lock`; `ci.yml`'s packaged Action consumer `version` and `trust.yml`'s `specsync-version`; every documented Action step in `README.md` and `site/src/content` (`@v6.4.1` with `version: '6.4.1'`); the CHANGELOG heading and the `[Unreleased]` and `[6.4.1]` links; the anchors for the version being released in `test-validate-release-candidate.py`.
- **Staying at published 6.4.0:** the `action.yml` default, `PUBLISHED_ACTION_DEFAULT`, the docs default row and current-install guidance. Prose about the immutable `@v6.4.0` tag keeps `version: '6.4.0'`.
- **The CHANGELOG** moves the `[Unreleased]` entries under `## [6.4.1] - 2026-11-21`, the planned tag day, corrected before the tag if the day moves.
- **`docs/releases/6.4.1.md`** has the sections of `docs/releases/6.4.0.md`. Its "Selected source" says the candidate is cut from `origin/main` at the commit Leif names with his go for the candidate tag.
