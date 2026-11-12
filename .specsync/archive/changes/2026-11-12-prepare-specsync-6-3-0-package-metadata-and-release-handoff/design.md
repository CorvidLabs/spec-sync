---
change: prepare-specsync-6-3-0-package-metadata-and-release-handoff
artifact: design
---

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
