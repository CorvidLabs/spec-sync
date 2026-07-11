---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: requirements
---

# Requirements

- Accepted evidence from a squash-merged branch remains trustworthy and archivable.
- The fallback must prove that the accepted workspace is already present unchanged on the remote default branch.
- Repository-relative Git tree lookups preserve that proof when the SpecSync project is nested in a monorepo.
- Unintegrated branches, changed scoped inputs, stale contracts, and mismatched approvals continue to fail closed.
- Release automation triggers only for exact semantic-version tags and verifies the tag matches Cargo metadata on main.
- The v5 Action downloads the 5.0.0 binary by default while allowing an explicit version override.
- Lifecycle digests use domain-separated, length-framed fields so embedded NUL bytes cannot alias file boundaries.
- Digest evidence distinguishes files, executable files, symlinks, and missing paths without normalizing binary bytes.
- Cross-platform topology tests isolate their temporary Git configuration from host line-ending defaults.
- The repository lifecycle stamp matches the 5.0.0 SDD layout written for new projects.
- The crates.io package contains only the CLI source and required user-facing metadata, excluding repository-only site,
  extension, test, spec, workflow, and agent assets.
- The README remains a concise entry point and links to current, complete documentation for detailed reference material.
