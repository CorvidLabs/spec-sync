---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: requirements
---

# Requirements

### REQ-github-006

Hosted verification SHALL assign each expensive confidence signal to one authoritative workflow,
while Trust SHALL retain release-binary identity, strict contract, risk, and provenance checks
without re-running the full product test suite.

Acceptance Criteria

- GitHub CI remains the authority for formatting, linting, full Rust tests, strict spec coverage,
  audit, coverage measurement, site, editor extension, and packaged-action consumer checks.
- `.trust.toml` invokes a dedicated `trust-lifecycle` lane that does not contain `cargo test`,
  clippy, or the full `verify` lane.
- `lanes.verify` remains the full local completion suite for agents and humans.
- Documentation identifies the current multi-OS matrix as Tier B work that requires a separately
  pinned protected-workflow update; this change does not silently weaken platform coverage.
- The thin PR contains no protected workflow files and no ship-status/product-code feature.
