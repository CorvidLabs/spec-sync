---
change: CHG-0042-prepare-and-publish-specsync-5-1-0-with-accurate-release-metadata-and-current-co
artifact: context
---

# Context

SpecSync 5.0.2 is the latest published release. The integrated 5.1 delivery work is
already represented by accepted CHG-0036, CHG-0038, CHG-0040, and CHG-0041,
including the final Windows portability follow-up merged through PR #384.

The release must preserve historical version references while updating only the
current package, lockfile, changelog, and Trust workflow surfaces. Fledge's release
dry run discovers `Cargo.toml` but not the explicit `specsync-version` input in
`.github/workflows/trust.yml`, so that workflow pin requires a deliberate edit.

The public comparison pages were last verified before Spec Kit 0.12.15, OpenSpec
1.6.0, and BMAD 6.10.0. They should distinguish deterministic blocking guarantees
from agent-led or optionally configured workflows and avoid implying that the four
products solve the same layer.
