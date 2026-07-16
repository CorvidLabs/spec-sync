---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: tasks
---

# Tasks

- [x] Merge PR #387 and validate its squash-integrated main tree.
- [x] Archive accepted CHG-0043 through CHG-0047.
- [x] Audit current GitHub Release, crates.io, Action, Homebrew, and Trust rollout state.
- [ ] Obtain explicit approval for the complete CHG-0048 definition.
- [ ] Start CHG-0048 and update all in-repository 5.1.1 metadata and public defaults.
- [ ] Add or update deterministic release/Action validation coverage.
- [ ] Pin Bun 1.3.14 in site deployment, site CI, and VS Code extension CI and verify all three
  workflows no longer require live Bun-tag discovery.
- [ ] Run the complete local release, strict, security, documentation, Action, Trust, and
  provenance matrix.
- [ ] Publish a draft release PR and clear hosted checks and review threads.
- [ ] Obtain explicit closing approval and accept CHG-0048.
- [ ] Merge the release PR and verify the exact integrated main commit.
- [ ] Publish and verify the immutable v5.1.1 GitHub release and crates.io crate.
- [ ] Promote and smoke-test the floating v5 Action ref.
- [ ] Update and test the Homebrew formula.
- [ ] Repair the dependent Trust rollout and complete clean installation smoke tests.
