---
id: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
state: archived
type: operations
base_commit: 85e6ae29b85792e4ed50af7417284ac755b6bd57
---

# Prepare the SpecSync 5.2.0 feature release: bump accurate release metadata and changelog, update the GitHub Action default to 5.2.0, document the native migrate 5.0 ledger backfill, batch correct-owner, inert registry stub tolerance, squash-merged archive trust, and legacy archive repair, verify all release artifacts and supported installation paths, and define fail-closed publication and rollback boundaries

## Intent

Prepare the SpecSync 5.2.0 feature release: bump accurate release metadata and changelog, update the GitHub Action default to 5.2.0, document the native migrate 5.0 ledger backfill, batch correct-owner, inert registry stub tolerance, squash-merged archive trust, and legacy archive repair, verify all release artifacts and supported installation paths, and define fail-closed publication and rollback boundaries

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- Cargo.toml and Cargo.lock read 5.2.0; CHANGELOG.md documents the 5.2.0 feature set (migrate 5.0 ledger backfill, batch correct-owner, inert registry stub tolerance, squash-merged archive trust, legacy archive repair); action.yml and all maintained workflow consumer pins read 5.2.0; README and site integration docs match; validate-release-version.py and validate-workflow-runtime-pins.py pass; full verification gate, specsync check --strict, and fledge trust verify pass; the change is accepted and merged before any tag or publication, and publication of v5.2.0 with rollback boundaries is defined but not executed by this change

## No-spec Rationale

Not applicable
