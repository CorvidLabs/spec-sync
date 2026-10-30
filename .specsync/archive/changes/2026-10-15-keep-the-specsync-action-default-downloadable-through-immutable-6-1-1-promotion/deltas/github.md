## MODIFIED

### REQUIREMENT REQ-github-002

The maintained GitHub Action SHALL expose immutable exact-version refs and a verified floating major compatibility ref. Its omitted-input binary default SHALL identify a published release with assets that has passed supported-platform verification. Before new stable assets are published, the default remains the last qualified published stable release, or a specifically qualified published release candidate for the package version under promotion. An unpublished package version SHALL NOT become the default merely because it matches Cargo metadata.

Stable promotion SHALL preserve the immutable qualified candidate SHA. An immutable Action tag may therefore retain the preceding published binary default; documentation SHALL identify its embedded default and show an explicit binary version for consumers requiring the tag's new stable binary. After new stable assets are published and exact-version Action smoke tests succeed, a separately verified main commit advances the default to the new stable binary. The floating major ref advances only to that verified commit. Immutable version tags SHALL NOT be retargeted.

Acceptance Criteria

- During preparation of `6.1.1`, the candidate Action default and inputs-table default stay `6.1.0`, the qualified published release with assets. Missing-asset stable or RC tags are never eligible defaults.
- Candidate consumers and Trust explicitly exercise version `6.1.1` through their runner-local candidate mirrors; the public omitted-input fallback never substitutes the published binary into candidate verification.
- One integrated immutable RC SHA is qualified on Ubuntu and macOS before final promotion. Its final `v6.1.1` tag and artifacts use that same SHA, without changing candidate content to advance the default.
- The immutable `v6.1.1` Action tag retains the candidate's `6.1.0` omitted-input default. Documentation tells exact-tag consumers to pass `version: '6.1.1'` after stable publication.
- Only after stable `6.1.1` assets and supported-platform Action smoke tests succeed does a separate verified main commit advance the default to `6.1.1`; floating `v6` then advances to that wrapper commit. A failed asset/checksum/Action smoke gate leaves the preceding default and floating ref unchanged.
- Immutable `v6.0.0` is not rewritten; its embedded default stays `6.0.0-rc.14`. Documentation tells consumers using that tag to pass `version: '6.0.0'`.
- The Action refuses Windows with an unsupported-platform message and WSL guidance rather than requesting unpublished Windows assets.
- Documentation distinguishes immutable pinning from the floating ref; the inputs-table default matches the downloadable default on main.
- The offline validator independently checks the declared published default and candidate package version. It rejects a premature `6.1.1`, arbitrary `9.9.9`, moving `latest`, missing default or docs drift while retaining explicit candidate version/mirror requirements.
