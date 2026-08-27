---
change: backfill-the-6-0-changelog-for-every-pr-from-630-onward-and-record-in-ci-confidence-md-what-the-release-lane-has
artifact: context
---

# Context

## What led here

Noticed while preparing rc.10: `grep` for recent issue references in `CHANGELOG.md` returned
**zero** matches for all twenty issues checked. The gap was not a few missing entries — it was
every PR merged after #627.

The cause is mundane and worth naming: entries were written *with* each change while the release
was young, and stopped being written once the work turned into a fast sequence of adopter-reported
fixes. Nothing enforces it. `change check` does not require a CHANGELOG entry, so the omission is
invisible until someone reads the file.

## What a session picking this up needs to know

**Entries were derived from diffs, not subject lines, and that was not ceremony.** Three issues
this release (#699, #706, #719) were filed with diagnoses that were directionally right and
specifically wrong, each caught only when an implementer re-derived from source. Writing 46 entries
from commit titles would have laundered every one of those errors into the release's public record.
Seven divergences were found this way; they are preserved as HTML comments next to their entries.

**Two findings escaped the changelog work and became their own issues or changes**, because they
were defects rather than descriptions:

- **#728** — `REQ-change-055` is live and unsuperseded but describes sequence allocation and
  `SPECSYNC_SEQUENCE_BASE`, all deleted by the ordinal retirement (#665). `AGENTS.md:49` still
  instructs agents to set the inert variable. The product half of that issue is the interesting
  half: `check` cannot see it, because a requirement whose implementation was *entirely* deleted
  has nothing left to measure against and so produces no finding at all. That is this release's
  most-repeated defect shape — **a category empty for want of input, read as a verdict** (#672,
  #684, #689's first design, #720) — appearing in the drift model itself rather than in one
  function.
- **The Windows qualification lane is failing**, and rc.8 and rc.9 are not valid candidates. The
  test target does not compile: one test in `src/commands/issues.rs` lacks the `#[cfg(unix)]` gate
  its fourteen siblings carry. Latent since #544. It surfaced only now because rc.1–rc.7 all died
  in `resolve` within 8–13 seconds, so `qualify` never ran — **fixing the outer gate revealed the
  failure it had been hiding.** Ordinary CI cannot catch this class: every job in `ci.yml` runs on
  `ubuntu-latest`, so Windows compiles only on a tag push.

## Ruled out

**Consolidating the pre-existing duplicate `### Fixed` and `### Changed` headings in
`[Unreleased]`.** They were already there. Fixing them means reordering roughly 700 lines of prose
nobody is changing, which would bury 46 new entries in an unreviewable diff. Left alone
deliberately; the section is correct, only untidy.
