## MODIFIED

### REQUIREMENT REQ-github-002

The maintained GitHub Action SHALL expose an immutable exact-version ref and a verified floating
major compatibility ref whose default binary version is synchronized only after exact-version
artifacts pass supported-platform verification. Before the stable release for the package version
under promotion is published, the composite Action's omitted-input default SHALL resolve to a
published GitHub Release that has assets (the latest qualified release candidate for that package
version). After the stable Release is published, the default matches the promoted stable package
version.

Acceptance Criteria

- Before the stable `vX.Y.Z` Release exists, the composite Action's current default equals a
  published release-candidate identifier for that `X.Y.Z` whose GitHub Release has assets (today
  `6.0.0-rc.14` for package `6.0.0`); tags without Release assets are not eligible defaults.
- After the stable Release is published, the composite Action's current default matches the
  promoted stable package version.
- An immutable `v<major>.<minor>.<patch>` Action ref resolves to the integrated release commit.
- The floating `v<major>` ref resolves to that same commit only after pinned consumers pass on
  Linux and macOS, the platforms SpecSync publishes binaries for as of 6.0.
- The Action refuses a Windows runner with a message naming the unsupported platform and WSL as
  the supported alternative, rather than requesting a release asset that is not published.
- Documentation distinguishes immutable pinning from the floating compatibility ref, and the
  Action inputs docs default matches the downloadable default.
- A failed exact-version asset or Action smoke test leaves the floating ref unchanged.

### SPEC SECTION Purpose

Links spec files to GitHub issues for traceability. Validates `implements` and `tracks` frontmatter
fields against actual GitHub issues, fetches issue metadata, and creates drift detection issues
when specs fall out of sync. Also defines the maintained composite GitHub Action distribution
contract: immutable exact-version refs and a verified floating major compatibility ref whose
downloadable default resolves to a published Release with assets — a qualified release candidate
while stable publication is pending, then the promoted stable release. Hosted JavaScript
verification uses one exact supported Bun runtime across site deployment, site CI, and VS Code
extension CI.
