---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: research
---

# Research

The 5.1.1 release (archived CHG-0048) established the exact surface list and order: archives
first (done in #406), then one release change covering `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md`, `action.yml`, README, `ci.yml`/`trust.yml` consumer pins, and the site
quickstart/integration docs, with `validate-release-version.py` enforcing cross-surface
consistency and `validate-workflow-runtime-pins.py` enforcing runtime pins. The same validators
already run in the lifecycle verification gate, so a consistent bump cannot regress silently.

All five shipped features are backward compatible (new command mode, new batch form of an
existing command, tolerance relaxations, and an archival trust widening), so the release is a
minor bump under the repository's semver practice. No spec Public API change affects the Action
interface; the `github` spec change is documentation-only via REQ-github-004.
