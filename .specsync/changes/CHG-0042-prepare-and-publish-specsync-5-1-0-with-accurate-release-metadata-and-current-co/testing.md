---
change: CHG-0042-prepare-and-publish-specsync-5-1-0-with-accurate-release-metadata-and-current-co
artifact: testing
---

# Testing

Validation must cover:

- `Cargo.toml` and `Cargo.lock` both resolve the local `specsync` package as 5.1.0.
- `.github/workflows/trust.yml` requests SpecSync 5.1.0.
- `fledge release 5.1.0 --dry-run --json` reports the expected release plan.
- `CHANGELOG.md` has valid 5.1.0 and comparison links with no accidental historical
  version rewrites.
- all active lifecycle records pass strict validation and only CHG-0042 remains
  active before its own acceptance/archive sequence.
- `fledge run fmt`, `lint`, `test`, `spec-check`, `docs-test`, `docs-lint`,
  `docs-build`, `vscode-compile`, `vscode-package`, `audit`, and `build` pass.
- `fledge trust verify` returns a proceed verdict and Attest provenance verifies on
  the exact release candidate commit.
- hosted Linux, macOS, Windows, CodeQL, packaged Action, documentation, coverage,
  audit, spec, and Trust checks pass before merge/tag.
