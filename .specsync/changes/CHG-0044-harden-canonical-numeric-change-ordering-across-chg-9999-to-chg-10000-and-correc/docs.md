---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: docs
---

# Docs

Update the unreleased 5.1 changelog entry for module JavaScript to mention both discovery/coverage and extensionless export-star resolution of sibling `.mjs` and `.cjs` files. Update both comparison-matrix headers in `site/src/content/docs/comparisons/adversarial-proof.md` from SpecSync 5.0 to 5.1.

Label the immutable Trust SHA as an unreleased candidate. No Trust v1.0.1 tag exists yet: SpecSync 5.1.0 is released first, then Trust 1.0.1 can compose it. CHG44 does not create a release, version bump, tag, or mutable channel.

The exact release head currently has valid provenance: the attestation note was pushed before CHG42 acceptance. That evidence remains a pre-tag boundary and is not replaced or fabricated by these documentation corrections.
