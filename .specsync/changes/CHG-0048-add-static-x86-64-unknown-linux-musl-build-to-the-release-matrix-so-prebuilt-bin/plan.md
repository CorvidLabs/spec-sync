---
change: CHG-0048-add-static-x86-64-unknown-linux-musl-build-to-the-release-matrix-so-prebuilt-bin
artifact: plan
---

# Plan

1. Add an `x86_64-unknown-linux-musl` matrix entry to `.github/workflows/release.yml` producing
   the `specsync-linux-x86_64-musl` artifact.
2. Add an `Install musl toolchain (Linux musl)` step gated on `contains(matrix.target, 'musl')`
   that installs `musl-tools` and exports `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER` and
   `CC_x86_64_unknown_linux_musl` as `musl-gcc`.
3. Add the musl binary to the Available Binaries table in
   `site/src/content/docs/integrations/github-action.md`.
4. Validate the workflow YAML and confirm the packaging, checksum, upload, and release-glob
   steps need no changes (they are artifact-name-driven).
