---
change: CHG-0048-add-static-x86-64-unknown-linux-musl-build-to-the-release-matrix-so-prebuilt-bin
artifact: testing
---

# Testing

The release workflow only runs on tag pushes, so the musl build is exercised end-to-end at the
next tag. Pre-merge verification is static and structural:

- Parse `.github/workflows/release.yml` as YAML to prove the matrix entry and toolchain step
  are syntactically valid.
- Confirm the musl matrix entry uses the same artifact-name-driven packaging, checksum,
  upload-artifact, and release-glob steps as every other target, so no step changes are needed.
- Confirm `ci.yml` and `trust.yml` build their own local mock `specsync-linux-x86_64` artifacts
  and are unaffected by the additional release artifact.
- Run the full `fledge trust verify` lane (fmt, clippy, build, spec-check, and tests) on the
  branch.
