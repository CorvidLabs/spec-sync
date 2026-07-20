---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: requirements
---

# Requirements

The 5.2.0 release preparation SHALL synchronize every in-repository version surface and keep
publication fail-closed.

- `Cargo.toml`, `Cargo.lock`, `action.yml`, and maintained consumer pins all read exactly 5.2.0.
- `CHANGELOG.md` names all five shipped features with their issue references.
- `validate-release-version.py` and `validate-workflow-runtime-pins.py` pass without edits to
  their logic.
- README and site docs show the 5.2.0 Action default and the new lifecycle commands.
- No tag, release, registry publish, or floating-ref move happens inside this change.
