---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: plan
---

# Plan

1. Archive squash-integrated CHG-0043 through CHG-0047 and prove strict lifecycle validity.
2. Define and approve CHG-0048 with exact version, Action compatibility, publication, parity, and
   rollback requirements.
3. Start implementation; bump Cargo/lock/changelog and synchronize Action, README, CI-consumer,
   Trust, and Action documentation version surfaces.
4. Pin Bun 1.3.14 across site deployment, site CI, and VS Code extension CI; add deterministic
   repository checks for runtime consistency, candidate version consistency, and the intended `v5`
   promotion contract where existing validation does not already cover them.
5. Run format, Clippy, complete unit/integration tests, strict specs, release build, documentation,
   audit, Action validation/consumer, Trust, and provenance verification.
6. Push a draft release PR, clear review feedback, and require the full hosted matrix on exact head.
7. Present evidence for explicit closing approval; accept CHG-0048 without publishing.
8. Merge the accepted release PR and require integrated-main validation.
9. Publish and verify `v5.1.1`, GitHub assets, crates.io, pinned/floating Actions, and Homebrew in
   monotonic order.
10. Repair the dependent Trust rollout, run clean installation smoke tests, and record final status.
