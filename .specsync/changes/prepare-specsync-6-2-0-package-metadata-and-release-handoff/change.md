---
id: prepare-specsync-6-2-0-package-metadata-and-release-handoff
state: implementing
type: operations
base_commit: 61e8ca6a1c83eb0597149d6a415c6e1355089080
---

# Prepare SpecSync 6.2.0 package metadata and release handoff

## Intent

Prepare SpecSync 6.2.0 package metadata and release handoff

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.2.0. The action.yml default, PUBLISHED_ACTION_DEFAULT and the Action docs default row stay published 6.1.2. README and site examples, archive-gate examples included, pin @v6.2.0 with version '6.2.0' for use after publication, and current-install guidance (ADOPTING, MIGRATION, site index) names published 6.1.2.
- CHANGELOG.md has a 6.2.0 section listing every user-visible change merged since v6.1.2 (#836, #837, #838, #840, #841, #853), with #842, #845 and #846 included and marked pending merge in the handoff, and the Unreleased link compares from v6.2.0. docs/releases/6.2.0.md names the selected source, what release.yml does and does not do on tag, and every remaining gate.
- validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass at 6.2.0, and the pre-push gate passes.
- No runtime source, canonical spec, dependency, workflow logic, tag or release changes. Before the RC tag, #842, #845 and #846 are either on main and merged into this branch or their CHANGELOG and handoff entries are removed.

## No-spec Rationale

Release package version, existing self-built candidate inputs and release documentation only; no runtime, canonical API, Action published default, risk or provenance policy changes.
