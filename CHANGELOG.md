# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-03-19

### Added

- **Spec generation documentation** — `specsync generate` now has a dedicated README section and expanded AI agents guide covering the full LLM workflow: generate → fill → validate → fix → enforce.
- Custom template documentation (`specs/_template.spec.md`) explaining how teams control generated spec structure.
- JSON output shape reference for `check` and `coverage` commands.
- Integration patterns table (pre-commit hooks, PR bots, CI coverage gates).

### Changed

- Rewrote README for density — every line carries new information, no filler.
- Elevated "For AI Agents" from a bullet list to a full "Spec Generation" section with usage examples and workflow guidance.
- Streamlined docs site pages to complement rather than duplicate the README.
- Updated CHANGELOG with previously missing 1.1.1 and 1.1.2 entries.

## [1.1.2] - 2026-03-19

### Fixed

- Resolved merge conflict markers in README.md.
- Removed overly broad permissions from CI workflow (code scanning alert fix).

### Changed

- Bumped `Cargo.toml` version to match the release tag.

## [1.1.1] - 2026-03-18

### Fixed

- Corrected GitHub Marketplace link after action rename.
- Renamed action from "SpecSync Check" to "SpecSync" for Marketplace consistency.
- Updated all marketplace URLs to reflect the new action name.

### Added

- GitHub Marketplace badge and link in README.

## [1.1.0] - 2026-03-18

### Added

- **Reusable GitHub Action** (`CorvidLabs/spec-sync@v1`) — auto-downloads the
  correct platform binary and runs specsync check in CI. Supports `strict`,
  `require-coverage`, `root`, and `version` inputs.
- **`watch` subcommand** — live spec validation that re-runs on file changes.
- **Comprehensive integration test suite** — end-to-end tests using assert_cmd.

### Changed

- Updated crates.io metadata (readme, homepage fields).

## [1.0.0] - 2026-03-18

### Added

- **Complete rewrite from TypeScript to Rust** for language-agnostic spec validation
  with significantly improved performance and a single static binary.
- **9 language backends** for export extraction: TypeScript/JavaScript, Rust, Go,
  Python, Swift, Kotlin, Java, C#, and Dart.
- **`check` command** — validates all spec files against source code, checking
  frontmatter, file existence, required sections, API surface coverage,
  DB table references, and dependency specs.
- **`coverage` command** — reports file and module coverage, listing unspecced
  files and modules.
- **`generate` command** — scaffolds spec files for unspecced modules using
  a customizable template (`_template.spec.md`).
- **`init` command** — creates a default `specsync.json` configuration file.
- **`--json` flag** — global CLI flag that outputs results as structured JSON
  instead of colored terminal text, for CI/CD and tooling integration.
- **`--strict` flag** — treats warnings as errors for CI enforcement.
- **`--require-coverage N` flag** — fails if file coverage percent is below
  the given threshold.
- **`--root` flag** — overrides the project root directory.
- **CI/CD workflows** with GitHub Actions for testing, linting, and
  multi-platform release binary publishing (Linux x86_64/aarch64,
  macOS x86_64/aarch64, Windows x86_64).
- Configurable required sections, exclude patterns, source extensions,
  and schema table validation via `specsync.json`.
- YAML frontmatter parsing without external YAML dependencies.
- API surface validation: detects undocumented exports (warnings) and
  phantom documentation for non-existent exports (errors).
- Dependency spec cross-referencing and Consumed By section validation.

[1.2.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.2.0
[1.1.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.2
[1.1.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.1
[1.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.0
[1.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.0.0
