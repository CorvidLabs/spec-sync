---
id: preserve-cargo-target-files-and-same-name-crate-roots-in-generated-specs
state: archived
type: bug_fix
base_commit: dd6eb3ef03de075556554c05b27476839da5c40a
---

# Preserve Cargo target files and same-name crate roots in generated specs

## Intent

Preserve Cargo target files and same-name crate roots in generated specs

## Affected Canonical Specs

- `manifest`

## Acceptance Criteria

- Explicit Cargo binary targets retain their exact source files and workspace prefixes, while a binary sharing its package name keeps the package source root. Generation binds same-name source modules together with crate roots and never assigns an explicit target to a library or guessed filename owner. Existing configured and spec ownership remain respected.

## No-spec Rationale

Not applicable
