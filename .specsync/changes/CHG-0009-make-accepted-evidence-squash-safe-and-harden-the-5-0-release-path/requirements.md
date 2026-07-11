---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: requirements
---

# Requirements

- Accepted evidence from a squash-merged branch remains trustworthy and archivable.
- The fallback must prove that the accepted workspace is already present unchanged on the remote default branch.
- Unintegrated branches, changed scoped inputs, stale contracts, and mismatched approvals continue to fail closed.
- Release automation triggers only for exact semantic-version tags and verifies the tag matches Cargo metadata on main.
- The v5 Action downloads the 5.0.0 binary by default while allowing an explicit version override.
