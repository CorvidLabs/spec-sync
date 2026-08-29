# Lesson bundle — remove-the-ship-date-from-the-6-0-0-changelog-heading-because-no-v6-0-0-tag-or-release-has-ever-existed

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Remove the ship date from the 6.0.0 changelog heading, because no v6.0.0 tag or release has ever existed
- **Kind**: Documentation
- **Paths**: CHANGELOG.md
- **Acceptance**: CHANGELOG.md contains a '## [6.0.0]' heading with no date, both validate-release-version.py checks still pass (the heading exists and the Unreleased compare link starts at v6.0.0), and a comment records why there is no date and that it must be added at the stable tag.

## Evidence

- Verification commit: `286439d8619b8edea4a95f354811d863ba512d2d`
- Base commit: `7df407728de3ac6458ef8807e79bbadb51da3324`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

## What led here

Found while triaging the open issue backlog. Not filed as an issue by anyone — it surfaced as an
operational note: the `[Unreleased]` section is 2,452 lines and `validate-release-version.py` only
asserts the `6.0.0` heading exists, so **CI cannot catch this**.

Looking at it turned up something narrower and worse than an unfolded section: the `6.0.0` heading
carried a date, and there is no v6.0.0 tag and no v6.0.0 release. Verified rather than assumed —
`git ls-remote --tags` has no `refs/tags/v6.0.0`, and `gh release list` shows only pre-releases.

## The near-miss worth recording

I first deleted the heading, reasoning that unreleased work belongs under `[Unreleased]` and a
dated heading for an unshipped version is simply wrong.

That would have broken the release lane. `validate-release-version.py:450` **requires** the
heading. It exists during the pre-release window on purpose, and I would have removed a guard while
believing I was removing a lie.

**The obvious repair was worse than the defect** — the same shape as #743, where adding
`&& scoped_review_current` would have made every squash-merging repository permanently unable to
finalize, and #741, where "always re-materialize" would destroy the reason the short-circuit exists.
In all three the fix that presents itself first is a larger regression than the bug. What caught it
here was reading the validator instead of trusting my reading of the problem.

## Ruled out

- **Folding `[Unreleased]` into `[6.0.0]` now.** Correct at the tag, unreviewable today: 2,452
  lines into 119, for a boundary that disappears within days.
- **Dating the heading with a planned release date.** That reintroduces the defect with a
  better-intentioned lie. The date is a fact about the past and belongs there when it is one.
- **Teaching the validator to reject a dated heading before its tag exists.** Defensible, and a real
  gap — the validator is the only thing reading this file mechanically and it checks presence, not
  truth. Out of scope here; worth its own issue if it recurs.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
