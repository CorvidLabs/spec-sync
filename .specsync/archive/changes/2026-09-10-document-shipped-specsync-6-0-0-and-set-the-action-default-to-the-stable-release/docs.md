---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: docs
---

# Docs

- README: GitHub Releases and crates.io serve 6.0.0; Homebrew tap is still 5.2.0; pin `@v6.0.0` plus `version: '6.0.0'`; no floating `v6` yet.
- Site github-action.md: default `6.0.0`; callout for the tagged Action's rc.14 leftover.
- Site index: `specsync check` first; SDD optional.
- ADOPTING: cargo install / GitHub tag; Trust 1.2.0.
- MIGRATION: Trust 1.2.0 defaults SpecSync 6.0.0; always pass Action `version`.
- RELEASING: 6.0.0 has shipped; remaining Homebrew bump and optional `v6` promotion; next patch is 6.0.1.
- SECURITY.md: keep the single `@v6` `uses:` form the validator requires; say the channel is unpublished and pin `@v6.0.0` until it exists.
- CHANGELOG: `## [6.0.0] - 2026-09-09`.
