---
change: add-a-hi-check-gate-to-ci
artifact: testing
---

# Testing

Verified before review:

- `hi check` passes locally on this branch: `159 criteria · 11 families · 11 files`,
  exit 0, under the pinned `hi` 0.4.0.
- The pin resolves: `human-intent` 0.4.0 is published on crates.io and ships a
  `Cargo.lock`, so `--locked` cannot fail for want of a lockfile.
- The workflow still parses as YAML and reports the expected job count.
- The repository's own CI meta-validators pass: `validate-workflow-runtime-pins.py` and
  `validate-release-version.py` both exit 0.
- The same step shape already ran green on a clean hosted runner in a sibling repository,
  so the install path is proven rather than assumed.

Verified by CI on this pull request:

- `hi-check` runs on a full run and reports success.
- `implementation-gate` lists `hi-check` among its dependencies.

Negative case worth exercising once by hand, since no automated test covers it here:
introduce a duplicate id into any `hi/*.md`, confirm `hi check` exits non-zero and that
`implementation-gate` consequently fails. Revert afterwards.
