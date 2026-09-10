---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: research
---

# Research

- `gh release view v6.0.0` is Latest, not prerelease, with linux/macos archives. Tag object `3c2ed497…`, peeled `b9ff3231`.
- `cargo search specsync` reports `6.0.0`. Homebrew `Formula/spec-sync.rb` is still `version "5.2.0"`.
- `git ls-remote` has no `refs/tags/v6`.
- Trust 1.2.0 (`fcc889f`) defaults `specsync-version` to `6.0.0`. Consumers omit the input or set `"6.0.0"`.
- `validate-release-version.py` `PUBLISHED_ACTION_DEFAULT = "6.0.0-rc.14"` with comment "set equal to the package version once v6.0.0 ships."
- REQ-github-002 already requires the omitted-input default to match the stable package version after the stable Release exists.
- SECURITY.md must contain exactly one `` `uses: CorvidLabs/spec-sync@v6` `` for the validator. The floating tag does not exist; surrounding prose must say pin `@v6.0.0` until it does.
