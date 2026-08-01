---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: context
---

# Context

CHG-0074 documents the approved confidence model but intentionally leaves protected workflows
unchanged. Today, `.github/workflows/ci.yml` runs Ubuntu, macOS, and Windows for ordinary pull
requests, while `.github/workflows/release.yml` starts only after a final `v*` tag already exists.
That spends cross-platform runner time during development and can discover a release failure after
the final tag has been published.

Trigger: the user approved Ubuntu as the ordinary integration authority and required cross-platform
validation before final release publication.

Root cause: development CI and release qualification share one always-on platform matrix, and no
immutable RC identity connects platform evidence to later final-tag creation.

Invariant: one immutable RC marker names one candidate commit; every required platform result and
the final release tag must resolve to that exact unchanged commit.

This is a protected-workflow change. It must use the repository's separately pinned required-workflow
process and must not be folded into CHG-0074.
