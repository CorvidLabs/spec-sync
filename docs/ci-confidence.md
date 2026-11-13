# CI confidence architecture (no duplicate suites)

This guide now lives on the documentation site:
[`site/src/content/docs/ci-confidence.md`](../site/src/content/docs/ci-confidence.md).

In short, each expensive confidence signal has one owner (REQ-github-006). GitHub CI owns
formatting, linting and the full test suite. The Trust action's lifecycle runs
`fledge lanes run trust-lifecycle`, which never re-runs that suite. `fledge lanes run verify` is the
full local suite. `.github/scripts/test-ci-lane-ownership.py` keeps it that way.
