---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: context
---

# Context

`v6.0.0` is a stable GitHub release (2026-09-09) at `b9ff3231`, crates.io serves `specsync 6.0.0`, and Trust 1.2.0 defaults SpecSync to 6.0.0. The tree still describes the candidate window: `action.yml` defaults to `6.0.0-rc.14`, `github-action.md` says stable is pending, CHANGELOG `## [6.0.0]` is undated with "stable publication is pending", RELEASING still treats crates.io as 5.2.0 and Homebrew bump as not done, README implies all channels may still be unpublished, and the site index leads with `change new` even though check is the product.

The tagged Action at `@v6.0.0` is immutable and still embeds default `6.0.0-rc.14`. This change updates `main` so the next Action tag defaults to 6.0.0, and tells consumers to always pass `version: '6.0.0'` on the shipped `@v6.0.0` tag.

Do not retag `v6.0.0`. Do not create floating `v6`. Homebrew tap is still 5.2.0; say so.
