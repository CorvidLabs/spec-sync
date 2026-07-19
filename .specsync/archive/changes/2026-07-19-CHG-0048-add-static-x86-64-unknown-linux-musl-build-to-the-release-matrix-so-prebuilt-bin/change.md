---
id: CHG-0048-add-static-x86-64-unknown-linux-musl-build-to-the-release-matrix-so-prebuilt-bin
state: archived
type: operations
base_commit: 4652ca1535eb65f7ba3ab0fc54b458b408fc174d
---

# Add static x86_64-unknown-linux-musl build to the release matrix so prebuilt binaries run on distros with glibc older than 2.39

## Intent

Add static x86_64-unknown-linux-musl build to the release matrix so prebuilt binaries run on distros with glibc older than 2.39

## Affected Canonical Specs

- None

## Acceptance Criteria

- Release workflow matrix includes an x86_64-unknown-linux-musl target producing a specsync-linux-x86_64-musl artifact with checksum through the existing artifact-name-driven packaging, upload, and release steps; the docs Available Binaries table lists the musl artifact.

## No-spec Rationale

CI release matrix and docs table only; no canonical spec module behavior changes
