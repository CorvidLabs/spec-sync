---
id: prepare-specsync-6-1-2-package-metadata-and-release-handoff
state: approved
type: operations
base_commit: f240d379b9f023f5766e5c9b39508eb84e03e2bb
---

# Prepare SpecSync 6.1.2 package metadata and release handoff

## Intent

Prepare SpecSync 6.1.2 package metadata and release handoff

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.toml and the specsync Cargo.lock entry, the CI packaged Action consumer version and the Trust specsync-version are 6.1.2. The action.yml default, PUBLISHED_ACTION_DEFAULT and the Action docs default row stay published 6.1.1. README and site examples pin @v6.1.2 with version '6.1.2', except archive-gate examples, which keep @main until 6.1.2 is published.
- CHANGELOG.md has a 6.1.2 section listing every user-visible change merged since v6.1.1 (#817, #818, #821 including RUSTSEC-2026-0381 and RUSTSEC-2026-0382, #823, #824, #828, #829, #832), with #830 marked pending merge, and the Unreleased link compares from v6.1.2. docs/releases/6.1.2.md names the selected source, what release.yml does and does not do on tag, and every remaining gate.
- validate-release-version.py, validate-workflow-runtime-pins.py and test-validate-release-candidate.py pass, the last including a control that publishing 6.1.2 ends the archive-gate @main allowance. cargo package --list --locked lists only the declared package files at 6.1.2.
- No runtime source, canonical spec, dependency, workflow logic, tag or release changes. Before the RC tag, #830 is either on main and merged into this branch or its CHANGELOG and handoff entries are removed.

## No-spec Rationale

Release package version, existing self-built candidate inputs and release documentation only; no runtime, canonical API, Action published default, risk or provenance policy changes.
