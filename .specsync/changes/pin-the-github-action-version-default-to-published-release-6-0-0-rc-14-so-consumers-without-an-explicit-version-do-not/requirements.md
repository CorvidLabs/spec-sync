---
change: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
artifact: requirements
---

# Requirements

`REQ-github-002` (modified) — the composite Action's downloadable `version` default SHALL
resolve to a published GitHub Release that has assets. Before the stable `vX.Y.Z` Release
exists, that default is the latest qualified release candidate with assets for the package
version under promotion (today `6.0.0-rc.14`). After the stable Release is published, the
default matches the promoted stable package version again.

Acceptance criteria additions:

- Omitting `version:` downloads a real release archive (no 404 on the default path).
- Tags without Release assets (`v6.0.0-rc.15`..`rc.17` today) are refused as the default.
- `validate-release-version.py` accepts the candidate-window Action/docs default when it is
  the pinned published candidate for the package version, while README immutable pin examples
  and mirrored CI consumer pins may still equal the package version.
