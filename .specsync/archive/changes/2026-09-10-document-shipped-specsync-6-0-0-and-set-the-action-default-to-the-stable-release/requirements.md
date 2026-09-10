---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: requirements
---

# Requirements

## Functional

### REQ-github-002

The maintained GitHub Action SHALL expose an immutable exact-version ref and a verified floating major compatibility ref whose default binary version is synchronized only after exact-version artifacts pass supported-platform verification. Before the stable release for the package version under promotion is published, the composite Action's omitted-input default SHALL resolve to a published GitHub Release that has assets (the latest qualified release candidate for that package version). After the stable Release is published, the default matches the promoted stable package version.

Acceptance Criteria

- Before the stable `vX.Y.Z` Release exists, the composite Action's current default equals a published release-candidate identifier for that `X.Y.Z` whose GitHub Release has assets. For the 6.0 candidate window that identifier was `6.0.0-rc.14`. Tags without Release assets are not eligible defaults.
- After the stable Release is published, the composite Action's current default on `main` matches the promoted stable package version (`6.0.0`).
- The immutable `v6.0.0` Action tag is not rewritten; its embedded default remains `6.0.0-rc.14`. Documentation tells consumers using that tag to pass `version: '6.0.0'`.
- An immutable `v<major>.<minor>.<patch>` Action ref resolves to the integrated release commit.
- The floating `v<major>` ref resolves to that same commit only after pinned consumers pass on Linux and macOS, the platforms SpecSync publishes binaries for as of 6.0. That ref does not exist yet.
- The Action refuses a Windows runner with a message naming the unsupported platform and WSL as the supported alternative, rather than requesting a release asset that is not published.
- Documentation distinguishes immutable pinning from the floating compatibility ref, and the Action inputs docs default matches the downloadable default on `main`.
- A failed exact-version asset or Action smoke test leaves the floating ref unchanged.
