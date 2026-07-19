---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: requirements
---

# Requirements

### REQ-github-002

The maintained GitHub Action SHALL expose an immutable exact-version ref and a verified floating
major compatibility ref whose default binary version is synchronized only after exact-version
artifacts pass supported-platform verification.

Acceptance Criteria

- The composite Action's current default matches the promoted stable package version.
- An immutable `v<major>.<minor>.<patch>` Action ref resolves to the integrated release commit.
- The floating `v<major>` ref resolves to that same commit only after pinned consumers pass on
  Linux, macOS, and Windows.
- Documentation distinguishes immutable pinning from the floating compatibility ref.
- A failed exact-version asset or Action smoke test leaves the floating ref unchanged.

### REQ-github-003

Hosted JavaScript verification SHALL select an exact supported Bun runtime rather than resolving
the newest Bun tag during each workflow run.

Acceptance Criteria

- Site deployment, site CI, and VS Code extension CI use the same exact Bun version.
- Setup does not query the Bun repository's live tag-discovery API to select a runtime version.
- The pinned runtime successfully installs frozen dependencies and passes the maintained site and
  extension verification commands.
