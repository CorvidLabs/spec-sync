---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: plan
---

# Plan

1. Define and approve CHG-0060 with exact version surfaces, promotion contract, and publication
   boundaries.
2. Bump Cargo/lock/changelog and synchronize Action, README, CI-consumer, and site documentation
   version surfaces to 5.2.0.
3. Run format, Clippy, complete unit/integration tests, strict specs, release validators, and
   the Trust lane.
4. Push a draft release PR and require the full hosted matrix on the exact head.
5. Accept after closing approval, merge, and leave tag/publication as the bounded next step.
