---
change: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
artifact: design
---

# Design

No new gates. Coordinated truth-up of the Action omitted-input default and user-facing docs.

- `action.yml` default and description become 6.0.0 (stable published, pin exact, Linux/macOS only).
- Validator `PUBLISHED_ACTION_DEFAULT` equals `6.0.0`.
- Site inputs table default becomes `6.0.0`, with a callout that the immutable `@v6.0.0` tag still embeds `6.0.0-rc.14` so consumers always pass `version: '6.0.0'`.
- Index/quickstart lead with `specsync check`; SDD is optional via `change adopt`.
- README states channel truth: GitHub Releases and crates.io are 6.0.0; Homebrew is still 5.2.0; no floating `v6`.
- Trust consumers: Trust 1.2.0, omit or set `specsync-version: "6.0.0"`.
- Pre-GA journals under `docs/6-0-*` stay historical.
