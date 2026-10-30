# Lesson bundle — preserve-cargo-target-files-and-same-name-crate-roots-in-generated-specs

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Preserve Cargo target files and same-name crate roots in generated specs
- **Kind**: BugFix
- **Specs**: manifest
- **Paths**: src/manifest/max_gradle_manifest_bytes.rs, src/manifest/tests.rs
- **Acceptance**: Explicit Cargo binary targets retain their exact source files and workspace prefixes, while a binary sharing its package name keeps the package source root. Generation binds same-name source modules together with crate roots and never assigns an explicit target to a library or guessed filename owner. Existing configured and spec ownership remain respected.

## Evidence

- Verification commit: `28779de1d407a5c35605298af92b3663a97709eb`
- Base commit: `dd6eb3ef03de075556554c05b27476839da5c40a`
- Verified by: `specsync check --spec manifest`

## From the change's context.md

# Context

Two automated PR #814 review findings reproduce against dd6eb3e. Package utils with src/lib.rs and src/utils/mod.rs drops lib.rs. Binary server at src/foo.rs binds src/lib.rs and duplicates the library owner. The user explicitly approved fixing issue #813 with local validation, confidence at least 95 percent, and no regressions. These are corrections to that same bug-fix request. The initial definition remains immutable; this narrow package owns only manifest parsing and parser tests. The reopened initial change owns generation, coverage and process regressions.

The fresh review of 5b640b7 reproduced four additional cases. This package extends the same target-identity correction to both conventional pathless binary layouts. The reopened generator package keeps explicit missing mappings authoritative and resolves shared targets or overlapping roots deterministically. The four added process regressions failed before these corrections and pass afterward. Human implementation review remains pending on the final updated diff.

## From the change's design.md

# Design

Retain exact bin target files in ManifestModule.source_paths while retaining their parent in source_dirs. Merge target paths with a same-name package rather than replacing its source root. Coverage counts exact file ownership; generation unions same-name sources, preserves configured mappings, and excludes exact targets from guessed aliases.

## From the change's testing.md

# Testing

Run cargo test --test integration generate_cargo, cargo test generator:: and cargo test manifest::. Run full fledge lanes run verify before merge. Cover directory/flat same-name modules, arbitrary and nested target paths, matching package/target names, both pathless binary defaults, workspace prefixes, no overwrite and existing ownership. Confirm the original issue and all review reproductions with the optimized binary.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-manifest-020` | `src/manifest/tests.rs`: `cargo_binary_target_paths_remain_exact_and_merge_same_name_package_roots` and `cargo_implicit_binary_paths_include_flat_and_nested_defaults` pass for exact targets, same-name package roots, both defaults and workspace prefixes. Eleven Cargo process regressions, 51 generator tests and 61 manifest tests pass, including differently named existing owners. Full local verification and optimized reproductions must pass again before completion. |

## Where these lessons go

- `specs/manifest/manifest.3md`
