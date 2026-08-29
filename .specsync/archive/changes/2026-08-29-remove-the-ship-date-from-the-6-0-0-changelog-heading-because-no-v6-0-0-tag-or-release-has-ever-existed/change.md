---
id: remove-the-ship-date-from-the-6-0-0-changelog-heading-because-no-v6-0-0-tag-or-release-has-ever-existed
state: archived
type: documentation
base_commit: 7df407728de3ac6458ef8807e79bbadb51da3324
---

# Remove the ship date from the 6.0.0 changelog heading, because no v6.0.0 tag or release has ever existed

## Intent

Remove the ship date from the 6.0.0 changelog heading, because no v6.0.0 tag or release has ever existed

## Affected Canonical Specs

- None

## Acceptance Criteria

- CHANGELOG.md contains a '## [6.0.0]' heading with no date, both validate-release-version.py checks still pass (the heading exists and the Unreleased compare link starts at v6.0.0), and a comment records why there is no date and that it must be added at the stable tag.

## No-spec Rationale

One heading line in a prose file; no canonical spec text changes and no requirement gains, loses or alters a guarantee.
