# Lesson bundle — record-homebrew-serving-6-0-0-after-the-tap-formula-bump

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Record Homebrew serving 6.0.0 after the tap formula bump
- **Kind**: Documentation
- **Paths**: README.md, MIGRATION.md, docs/ADOPTING.md, docs/RELEASING.md, docs/ci-confidence.md, site/src/content/docs/quickstart.md
- **Acceptance**: Every present-tense claim that the Homebrew tap serves SpecSync 5.2.0 is gone from README.md, MIGRATION.md, docs/ADOPTING.md, docs/RELEASING.md, docs/ci-confidence.md, and site/src/content/docs/quickstart.md. Those documents now say Homebrew serves 6.0.0, which matches CorvidLabs/homebrew-tap Formula/spec-sync.rb at version 6.0.0 on main. docs/RELEASING.md additionally records that Formula/corvid-trust.rb asserts its dependency versions in its test block, so the two formulae must be bumped in the same change. Historical records under docs/6-0-*, docs/GOAL-*, docs/superpowers, specs/, and CHANGELOG.md are left untouched.

## Evidence

- Verification commit: `177a28e9f6af797250fa8906dfaafee1bdc51781`
- Base commit: `0188edba726e738190468a90bb17070a667996e0`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

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

## From the change's design.md

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

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
