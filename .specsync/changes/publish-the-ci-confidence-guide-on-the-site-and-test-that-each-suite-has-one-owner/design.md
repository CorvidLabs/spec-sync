---
change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
artifact: design
---

# Design

- `test-ci-lane-ownership.py` reads both TOML files with `tomllib`. `ownership_errors` checks that
  the Trust lifecycle command is `fledge lanes run trust-lifecycle`, that the lane runs no
  `test`, `test-nextest`, `lint` or `verify` step and no task whose command runs `cargo test`,
  clippy or the verify lane, and that `lanes.verify` still runs fmt, lint, check-types, test, build
  and spec-check. Five mutation tests prove each rule bites.
- A Fledge task `ci-lane-ownership-test` and a CI step in `validate-action` run it.
