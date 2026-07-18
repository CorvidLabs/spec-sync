---
change: CHG-0046-make-lifecycle-verification-workflows-evaluate-the-exact-pull-request-head-while
artifact: design
---

# Design

Set `ref` on exactly two checkout steps: the `spec-check` job in `ci.yml` and the `trust`
job in `trust.yml`. The expression selects `github.event.pull_request.head.sha` for a
`pull_request` event and falls back to `github.sha` for `push`. Both retain `fetch-depth: 0`
so ancestry-sensitive lifecycle and provenance checks have complete history.

No other checkout step changes. Build, test, formatting, audit, coverage, packaged-action,
site, extension, reporting, and attestation jobs keep GitHub's default synthetic-merge
checkout behavior on pull requests.
