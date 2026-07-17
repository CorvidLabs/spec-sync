---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: tasks
---

# Tasks

- [x] Merge PR #387 and validate its squash-integrated main tree.
- [x] Archive accepted CHG-0043 through CHG-0047.
- [x] Audit current GitHub Release, crates.io, Action, Homebrew, and Trust rollout state.
- [x] Obtain explicit approval for the complete CHG-0048 definition.
- [x] Start CHG-0048 and update all in-repository 5.1.1 metadata and public defaults.
- [x] Add or update deterministic release/Action validation coverage.
- [x] Pin Bun 1.3.14 in site deployment, site CI, and VS Code extension CI and verify all three
  workflows no longer require live Bun-tag discovery.
- [x] Run pre-verification format, lint, unit/integration, documentation, audit, release-build,
  packaged-Action, strict coverage, and release-package checks; confirm hosted CI, final Trust and
  provenance, closing approval, integration, publication, and distribution smoke tests remain
  fail-closed delivery gates after lifecycle verification.
- [x] Address all PR #389 review findings: harden malformed-YAML diagnostics, run release guards
  for every validated surface, persist validator commands in lifecycle evidence, and keep public
  examples on the immutable Action ref until floating-channel promotion.
