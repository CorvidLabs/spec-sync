## MODIFIED

### REQUIREMENT REQ-github-002

The maintained GitHub Action SHALL expose an immutable exact-version ref and a verified floating major compatibility ref whose default binary version is synchronized only after exact-version artifacts pass supported-platform verification. Before the stable release for the package version under promotion is published, the composite Action's omitted-input default SHALL resolve to a published GitHub Release that has assets (the latest qualified release candidate for that package version). After the stable Release is published, the default matches the promoted stable package version.

Acceptance Criteria

- Before the stable `vX.Y.Z` Release exists, the composite Action's current default equals a published release-candidate identifier for that `X.Y.Z` whose GitHub Release has assets. For the 6.0 candidate window that identifier was `6.0.0-rc.14`. Tags without Release assets are not eligible defaults.
- After the stable Release is published, the composite Action's current default on `main` matches the promoted stable package version (`6.1.1`).
- The immutable `v6.0.0` Action tag is not rewritten; its embedded default remains `6.0.0-rc.14`. Documentation tells consumers using that tag to pass `version: '6.0.0'`.
- An immutable `v<major>.<minor>.<patch>` Action ref resolves to the integrated release commit.
- The floating `v6` ref resolves to a commit whose composite Action omitted-input default is `6.1.1`. It SHALL NOT resolve to immutable tag `v6.0.0`, whose embedded default remains `6.0.0-rc.14`.
- The Action refuses a Windows runner with a message naming the unsupported platform and WSL as the supported alternative, rather than requesting a release asset that is not published.
- Documentation distinguishes immutable pinning from the floating compatibility ref, and the Action inputs docs default matches the downloadable default on `main`.
- A failed exact-version asset or Action smoke test leaves the floating ref unchanged.

## MODIFIED

### SPEC SECTION Purpose

Links spec files to GitHub issues for traceability. Validates `implements` and `tracks` frontmatter
fields against actual GitHub issues, fetches issue metadata, and creates drift detection issues
when specs fall out of sync. Also defines the maintained composite GitHub Action distribution
contract: immutable exact-version refs and a verified floating major compatibility ref. After
stable `v6.1.1` is published, the omitted-input default on `main` and on floating `@v6` is
`6.1.1`. The published `@v6.0.0` Action tag still embeds default `6.0.0-rc.14` and is not
rewritten; `v6` must not be retargeted at that tag. Hosted JavaScript
verification uses one exact supported Bun runtime across site deployment, site CI, and VS Code
extension CI.
