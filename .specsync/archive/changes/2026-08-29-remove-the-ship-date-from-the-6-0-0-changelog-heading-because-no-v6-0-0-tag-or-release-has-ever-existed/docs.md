---
change: remove-the-ship-date-from-the-6-0-0-changelog-heading-because-no-v6-0-0-tag-or-release-has-ever-existed
artifact: docs
---

# Docs

`CHANGELOG.md` carried `## [6.0.0] - 2026-07-29` — a **dated release heading for a version that has
never been tagged or released.** Every 6.0 tag is a pre-release; the latest is `v6.0.0-rc.10`.
2026-07-29 was when 6.0 work began. To any reader, and to anything that parses this file, it
claimed a ship date.

The date is removed. The heading stays.

## Why the heading stays

`.github/scripts/validate-release-version.py:450` requires `## [{version}]` to exist whenever the
crate version is 6.0.0:

    if f"## [{version}]" not in changelog:
        errors.append(f"CHANGELOG.md must contain a {version} release section")

So the heading is **mandatory throughout the pre-release window**, by design. It is not a mistake.
Only the date was false.

**My first attempt deleted the heading outright, which would have broken the release lane.** Reading
the validator is what caught it — not the tests, which do not run that script, and not CI, which
would have failed at release time with a message about a missing changelog section rather than about
the heading someone removed on purpose.

Keep a Changelog dates released versions only, so an undated heading is exactly right for something
not yet shipped. A comment above it records why there is no date and that the date is added at the
stable tag, so the next reader does not helpfully restore one.

## Not done here

The `[Unreleased]` section is 2,452 lines and sits above this heading. **Both are the same
unreleased release.** Folding them is correct at the stable tag and wrong now: it would produce a
diff nobody can review, for a boundary that disappears in a few days. The comment says so, so the
arrangement is deliberate rather than abandoned.
