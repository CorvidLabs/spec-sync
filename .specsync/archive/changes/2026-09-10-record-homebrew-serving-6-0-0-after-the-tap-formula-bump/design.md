---
change: record-homebrew-serving-6-0-0-after-the-tap-formula-bump
artifact: design
---

# Design

Documentation only. No code, no spec text, no behavior.

## Rule applied

A statement about where SpecSync can be installed is a claim about the world.
It is corrected when the world changes. A statement about what was true during a
past release is a record. It is preserved.

Every edit below is the first kind. Everything left alone is the second.

## Edits

| File | Was | Now |
| --- | --- | --- |
| `README.md` | Releases and crates.io serve 6.0.0, Homebrew still serves 5.2.0 until the tap is bumped | Releases, crates.io and Homebrew all serve 6.0.0 |
| `MIGRATION.md` | "Homebrew still serves 5.2.0." | "Homebrew (`CorvidLabs/tap/spec-sync`) serves 6.0.0." |
| `docs/ADOPTING.md` | do not use the tap for 6.0.0 until the formula is bumped | the tap serves 6.0.0; the Linux and macOS only note is kept |
| `docs/RELEASING.md` | Homebrew still serves 5.2.0; section 5 is an immediate catch-up | Homebrew serves 6.0.0; the catch-up has landed via homebrew-tap#27 |
| `docs/ci-confidence.md` | the tap still serves 5.2.0 | all three channels serve 6.0.0, with the lag noted as the reason the lesson exists |
| `site/src/content/docs/quickstart.md` | inline caveat on the `brew install` line | caveat removed |

## Addition

`docs/RELEASING.md` gains a short note that `Formula/corvid-trust.rb` asserts its
dependency versions in its `test do` block, so `spec-sync` and `corvid-trust`
must be bumped in the same change or `brew test corvid-trust` breaks. This was
discovered while making the bump and is not recorded anywhere else.

## Not changed

`CHANGELOG.md`, `docs/6-0-*`, `docs/GOAL-*`, `docs/superpowers/**`, `specs/**`.
All historical. The Windows and no-floating-`v6` statements are untouched because
both are still true.
