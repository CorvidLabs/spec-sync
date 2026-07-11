---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: testing
---

# Testing

Every added requirement ID must appear in this evidence plan.

## Static contract gates

- Run strict checking with 100% file coverage and zero warnings.
- Run `specsync score --all`; require every spec to remain grade A.
- Run `specsync deps --strict --format json`; require no undeclared imports, missing dependencies, or cycles.
- Audit all 62 requirements companions for a module-matching `REQ` ID, SHALL statement, and Acceptance Criteria.
- Audit all 62 companion sets for matching frontmatter and nonempty required files.

## Focused behavior

- Run the complete `commands::migrate` unit suite and every `migrate_*` integration fixture.
- Run configuration serialization and migration-refusal fixtures, including an exact assertion that canonical TOML
  begins with `# spec-sync configuration`.
- Validate the nine corrected Public API parameter cells against their source declarations.
- Validate the 35 dependency edges against source imports.
- Add Rust dependency fixtures for comments, quoted/raw/byte/raw-byte strings, and source-owner resolution.
- Run rehash command behavior after removing its dependency on the parent command registry.

## Full gates

- Run Rust formatting, Clippy with warnings denied, the complete unit/integration suite, coverage, RustSec audit,
  documentation tests/build, VS Code packaging, Action consumer tests, and crate dry-run packaging.
- Run configured lifecycle verification and retain requirement evidence for every CHG-0010 delta ID.
- Require the Linux, macOS, and Windows PR matrix plus post-merge main to pass before release.

## Requirement traceability

All requirement IDs introduced by the 44 migration deltas are validated by the requirements-companion audit and strict
effective-contract check. API, configuration, dependency, and maturity corrections are validated by the focused gates
above and the module tests named in their canonical `testing.md` companions.

Evidence covers:

- `REQ-archive-001`, `REQ-changelog-001`, `REQ-compact-001`, `REQ-deps-001`, `REQ-git-utils-001`,
  `REQ-github-001`, `REQ-hash-cache-001`, `REQ-hooks-001`, `REQ-ignore-001`, `REQ-importer-001`,
  `REQ-manifest-001`, `REQ-merge-001`, `REQ-output-001`, `REQ-parser-001`, `REQ-registry-001`,
  `REQ-rehash-001`, `REQ-schema-001`, `REQ-scoring-001`, `REQ-util-001`, `REQ-validator-001`,
  `REQ-view-001`, and `REQ-watch-001`.
- `REQ-cmd-archive-tasks-001`, `REQ-cmd-changelog-001`, `REQ-cmd-compact-001`, `REQ-cmd-coverage-001`,
  `REQ-cmd-deps-001`, `REQ-cmd-diff-001`, `REQ-cmd-hooks-001`, `REQ-cmd-import-001`,
  `REQ-cmd-init-registry-001`, `REQ-cmd-issues-001`, `REQ-cmd-lifecycle-001`, `REQ-cmd-merge-001`,
  `REQ-cmd-migrate-001`, `REQ-cmd-new-001`, `REQ-cmd-report-001`, `REQ-cmd-resolve-001`,
  `REQ-cmd-rules-001`, `REQ-cmd-scaffold-001`, `REQ-cmd-score-001`, `REQ-cmd-stale-001`,
  `REQ-cmd-view-001`, and `REQ-cmd-wizard-001`.

## Local results

- Effective lifecycle validation passes for both active changes with zero errors or warnings.
- Dependency validation passes across 62 modules and 215 edges with zero cycles, missing dependencies, undeclared
  imports, errors, or warnings.
- The complete Rust suite passes: 1,526 unit tests and 187 integration tests, zero failures. Formatting and Clippy
  with warnings denied also pass.
- Focused results pass: 81 lifecycle tests, 24 dependency-analysis tests, two rehash tests, and the configuration
  header regression.
- Documentation tests (23), Astro diagnostics/build (38 pages), VS Code compilation/package, and release workflow
  `actionlint` all pass locally.
- The crates.io allowlist resolves to 113 entries and excludes repository-only assets. RustSec refresh and crates.io
  dry-run publication are pending CI because this sandbox cannot write the external advisory cache or reach the index.
- Linux, macOS, and Windows CI plus canonical post-acceptance scoring remain mandatory before release.
