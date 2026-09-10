---
change: record-homebrew-serving-6-0-0-after-the-tap-formula-bump
artifact: docs
---

# Docs

This change is entirely documentation, so the docs artifact is the change.

## Reader-visible effect

Anyone following an install path is no longer steered away from Homebrew.
Before, all four entry points (`README.md`, `docs/ADOPTING.md`, the site
quickstart, and `MIGRATION.md`) told a reader that `brew install` would hand
them 5.2.0 and that they should use cargo or a release asset instead. That is no
longer true, and following it now costs a reader the simplest install path for
no reason.

The three install channels are now described consistently:

- `cargo install specsync` serves 6.0.0
- `brew install CorvidLabs/tap/spec-sync` serves 6.0.0
- GitHub Release assets serve 6.0.0

## Statements deliberately kept

These are unchanged because they are still accurate:

- Linux and macOS only. 6.0 publishes no Windows binary (#735). Windows users
  run under WSL or build from source.
- There is no floating `v6` tag.
- The immutable `@v6.0.0` Action tag still defaults an omitted `version` input to
  `6.0.0-rc.14`, so consumers must pass `version: '6.0.0'` explicitly. This is
  the single most load-bearing warning in the install documentation and is not
  touched by this change.

## Cross-repository follow-up

`CorvidLabs/site` carries the same caveat in two places that are outside this
repository: `src/pages/spec-sync/docs/_content/quickstart.md` and the 6.0.0 blog
post `src/pages/blog/specsync-six-check-is-the-product.md`. Those are handled in
a separate pull request against that repository.
