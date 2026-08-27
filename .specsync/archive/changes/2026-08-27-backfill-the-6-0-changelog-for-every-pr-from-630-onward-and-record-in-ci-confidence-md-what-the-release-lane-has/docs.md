---
change: backfill-the-6-0-changelog-for-every-pr-from-630-onward-and-record-in-ci-confidence-md-what-the-release-lane-has
artifact: docs
---

# Docs

Two documentation surfaces change, both adopter-facing.

## `CHANGELOG.md`

The `[Unreleased]` section covered 6.0 work through PR #627 and then stopped. Every PR from
**#630 through #727** — 46 merges, ~52 issues, the entire back half of the release — had no entry.
That included every defect an adopter actually reported against a release candidate.

46 entries are added, grouped under `Added`, `Changed`, `Security` and `Fixed`. The existing
`Removed` entry (the Windows binary) stays first: it is the single most important thing an
upgrader needs, and burying it under 1,700 lines of newer prose would be a regression in the
document's job.

Each entry was derived from the commit's **diff and issue thread**, never from its subject line.
That distinction produced seven recorded divergences, kept as HTML comments beside the entries
they explain — invisible when rendered, available to anyone doing archaeology later. Two are worth
naming here because the subject line is what a reader would otherwise trust:

- **#668** ("the lane must be able to read the tag that triggered it") did not do that. Its whole
  diff is `fetch-tags: true` on `actions/checkout`, which is a no-op when `fetch-depth: 0` — the
  action assigns `fetchTags` only inside its `fetchDepth > 0` branch. #669 established this 39
  minutes later and replaced it with an explicit `git fetch --force`. The entry credits #669.
- **#715** ("one canonical frontmatter reader") unified four *strippers*, not every reader. Two
  non-canonical readers survive on `main` — `registry.rs`'s line-wise `extract_module_name` and
  `commands/lifecycle.rs`'s unanchored `find("---\n")`. The commit *body* says so; the subject
  does not.

## `docs/ci-confidence.md`

Its "Tag authority" section reasons carefully about what the release lane enforces, and never says
which of it has ever **run**. A new subsection records that: `resolve` and `validate` executed for
real (the `workflow_dispatch` dry run on 27 Aug, the first in the repository's history), `qualify`
executed for the first time on rc.8 and is failing on Windows, and `promote` has **never executed**.

`promote` cannot be rehearsed here — `final_tag` derives from the candidate's own `Cargo.toml`, so
any run against a real candidate mints the real tag. What was proven instead is stated with its
limits: the git mechanics were transcribed verbatim against a local bare remote and all three
branches exercised, and both rulesets were confirmed to carry `update` and `deletion` only, so tag
*creation* is unrestricted and immutability cannot block the mint. What remains unproven is named
rather than glossed: a local path remote never asks for a credential, so the rehearsal proves the
credential helper's syntax does not break the invocation, not that it authenticates.
