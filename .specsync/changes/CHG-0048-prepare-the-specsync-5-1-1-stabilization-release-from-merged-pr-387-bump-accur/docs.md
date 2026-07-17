---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: docs
---

# Docs

Update only current installation guidance:

- `README.md` keeps `@v5` as the compatible 5.x path after that ref exists and updates the
  immutable example to `@v5.1.1` with `version: '5.1.1'`.
- `site/src/content/docs/integrations/github-action.md` reports the real 5.1.1 default and explains
  the difference between immutable `@v5.1.1` and floating `@v5` usage.
- `CHANGELOG.md` adds a 5.1.1 patch section covering lifecycle evidence correctness, strict-check
  performance, security/log handling, and Windows Git/path portability without reclassifying
  accepted 5.1.0 features.
- GitHub release notes identify the release as post-5.1.0 stabilization and link the verified
  installation surfaces.

Do not announce Homebrew, crates.io, or `@v5` availability until each corresponding publication and
smoke check succeeds. Historical documentation and archived lifecycle evidence remain unchanged.
