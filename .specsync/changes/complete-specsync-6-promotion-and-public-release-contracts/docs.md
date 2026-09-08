---
change: complete-specsync-6-promotion-and-public-release-contracts
artifact: docs
---

# Docs

Reviewable scope:
- release.yml: correct both evidence cardinality guards only; preserve release protections.
- test-validate-release-candidate.py: exercise actual guards with the authoritative supported count and negative controls.
- validate-release-candidate.py: remove obsolete Windows filenames from current two-platform diagnostics only.
- src/cli.rs: describe scoped human review and reviewer claims without asserting local authentication.
- cli_args/change canonical specs, requirements, context, tasks: reflect the actual claim/provenance boundary and preserve all existing review/freshness protections.
- README.md: use slug IDs, include review, distinguish materialization from finalization, fix workspace paths, and retain the structural validation boundary.
- site CLI reference: use --spec and required --digest for supersede; distinguish normal/recovery workflows and supported reviewer identity claims.
- SECURITY.md: explicitly support6.x including candidates, retain current5.x status.
- CHANGELOG.md: fold6.0 entries under one6.0 release section; no invented publication date.
- docs/ci-confidence.md: align current topology with Ubuntu/macOS and document exact qualification, dry-run, package-channel, and hub-deploy evidence requirements.

Do not add automatic crates.io/Homebrew publishing, change release protection settings, or implement a new authenticated-identity subsystem in this patch.
