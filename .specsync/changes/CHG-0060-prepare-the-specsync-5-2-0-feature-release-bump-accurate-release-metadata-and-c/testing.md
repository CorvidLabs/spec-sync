---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: testing
---

# Testing

- `REQ-github-004`: `validate-release-version.py` passes with every surface at exactly 5.2.0,
  proving the Action default and maintained consumer pins synchronize on the exact version, and
  the floating-ref promotion rule is documented in the github canonical spec.
- `validate-workflow-runtime-pins.py` passes unchanged.
- Full `cargo test`, `cargo fmt --check`, Clippy, and `specsync check --strict` pass on the
  release tree.
- `fledge trust verify` passes on the release branch before the draft PR is marked ready.
