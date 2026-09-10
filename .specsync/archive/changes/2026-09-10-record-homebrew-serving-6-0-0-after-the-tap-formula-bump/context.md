---
change: record-homebrew-serving-6-0-0-after-the-tap-formula-bump
artifact: context
---

# Context

## What led here

SpecSync 6.0.0 shipped on 2026-09-09. GitHub Latest and crates.io both moved to
6.0.0 immediately, but the Homebrew tap did not: `CorvidLabs/homebrew-tap`
`Formula/spec-sync.rb` stayed pinned at 5.2.0, a full major version behind.

The documentation handled that correctly at the time. `README.md`,
`MIGRATION.md`, `docs/ADOPTING.md`, `docs/RELEASING.md` and
`docs/ci-confidence.md` each carried an explicit caveat telling readers the tap
still served 5.2.0 and to install from cargo or a GitHub Release asset instead.
`docs/RELEASING.md` additionally recorded the bump as an immediate catch-up.

That gap has now been closed. homebrew-tap#27 bumped `Formula/spec-sync.rb` to
6.0.0 and `Formula/corvid-trust.rb` to 1.2.0, and it is merged. `brew install
CorvidLabs/tap/spec-sync` now installs 6.0.0.

The caveats are therefore stale in the other direction: they now tell readers to
avoid a channel that is correct, and they understate where SpecSync 6.0.0 is
available.

## Constraints a session picking this up needs to know

The two formulae are coupled. `Formula/corvid-trust.rb` asserts its dependency
versions inside its `test do` block, including
`assert_match "5.2.0", shell_output("specsync --version")`. Bumping `spec-sync`
alone would have left that assertion pointing at the old version and broken
`brew test corvid-trust`. That coupling is not obvious from either formula in
isolation, so this change records it in `docs/RELEASING.md` for the next release.

## Ruled out

Rewriting historical records. `CHANGELOG.md`, `docs/6-0-*`, `docs/GOAL-*`,
`docs/superpowers/**` and everything under `specs/` describe the state at the
time they were written. A caveat that was true when recorded is history, not
drift, and rewriting it would falsify the record.

`docs/ci-confidence.md` is a borderline case. It is a point-in-time confidence
report, but the sentence in question is written in the present tense and its
surrounding lesson, do not infer package availability on every advertised
channel from a source tag, is still worth keeping. It is reworded to stay
accurate while preserving that lesson, rather than deleted.
