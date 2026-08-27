## MODIFIED

### REQUIREMENT REQ-github-002

The maintained GitHub Action SHALL expose an immutable exact-version ref and a verified floating
major compatibility ref whose default binary version is synchronized only after exact-version
artifacts pass supported-platform verification.

Acceptance Criteria

- The composite Action's current default matches the promoted stable package version.
- An immutable `v<major>.<minor>.<patch>` Action ref resolves to the integrated release commit.
- The floating `v<major>` ref resolves to that same commit only after pinned consumers pass on
  Linux and macOS, the platforms SpecSync publishes binaries for as of 6.0.
- The Action refuses a Windows runner with a message naming the unsupported platform and WSL as
  the supported alternative, rather than requesting a release asset that is not published.
- Documentation distinguishes immutable pinning from the floating compatibility ref.
- A failed exact-version asset or Action smoke test leaves the floating ref unchanged.
