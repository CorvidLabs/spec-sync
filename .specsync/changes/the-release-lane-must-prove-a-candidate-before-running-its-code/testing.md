---
change: the-release-lane-must-prove-a-candidate-before-running-its-code
artifact: testing
---

# Testing

The release lane cannot be exercised without tagging — which is the thing this change exists to
make safe — so the checks are static and were run against the edited file:

- `release.yml` parses as YAML.
- In `validate`, `merge-base --is-ancestor` precedes the first `cargo` invocation.
- Both `rust-cache` steps carry `save-if: false`.
- `authorize-release` contains exactly one `uses:` — its checkout — and no caching step, which
  is the condition that makes keeping that checkout tolerable. Checked with comment lines
  excluded: the first attempt at this check matched its own comment text and reported three
  caching steps that do not exist.

The lane's `dry_run` dispatch input is the intended way to exercise this before a real tag.
