# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0] - 2026-04-11

### Breaking Changes

- **Directory restructure** — all spec-sync metadata moves into `.specsync/`: config, registry, lifecycle history, change records, and archives. Root-level `specsync.json` and `specsync-registry.toml` are relocated automatically by `specsync migrate`.
- **Config format change** — `specsync.json` is converted to `.specsync/config.toml` (TOML). Legacy JSON/TOML files at the root still work as fallback.
- **`lifecycle_log` removed from frontmatter** — lifecycle history is extracted from spec YAML frontmatter into `.specsync/lifecycle/*.json` files. The `lifecycle_log` field is removed from specs during migration.
- **GitHub Action version** — update workflows from `@v3` to `@v4`.

### Added

- **`specsync migrate` command** — automated 3.x → 4.0.0 migration with 10 steps: version detection, backup, directory creation, config conversion, registry relocation, lifecycle extraction, frontmatter cleanup, gitignore, cross-project ref scanning, and version stamping. Supports `--dry-run`, `--no-backup`, `--format json`. Idempotent and safe to re-run (#198).
- **Full spec lifecycle management** — `specsync lifecycle` subcommands: `status`, `promote`, `demote`, `set`, `history`, `guard`, `auto-promote`, `enforce`. Specs track lifecycle stages (draft → review → stable → deprecated → archived) with configurable transition guards.
- **Lifecycle enforcement in CI** — `specsync lifecycle enforce --all` validates lifecycle rules in CI. Available via GitHub Action with `lifecycle-enforce: 'true'`.
- **Change records** — `.specsync/changes/` directory for tracking spec modifications over time.
- **Spec archival** — `.specsync/archive/` directory for retired specs. Archive contents are version-controlled (not gitignored).
- **Migration backup** — `.specsync/backup-3x/` with timestamped manifest preserves original 3.x files for rollback.
- **Cross-project reference scanning** — migration detects `depends_on` refs to external repos and records them in `.specsync/cross-project-refs.json`.

### Fixed

- **Archive not gitignored** — `.specsync/archive/` is no longer excluded from git. Users who want to remove archived specs can delete them explicitly (#202).

### Documentation

- **MIGRATION.md** — comprehensive upgrade guide with breaking changes, step-by-step instructions, and FAQ.

## [3.8.0] - 2026-04-10

### Added

- **Staleness detection** — new `specsync stale` command identifies specs that haven't been updated since their source files changed. Also available via `specsync check --stale` (#189).
- **AST-based export parsing** — tree-sitter powered export extraction replaces regex-based parsing for more accurate and reliable results across all supported languages (#192).
- **Batch operations** — `specsync import --all-issues` and `--from-dir` for bulk import; `specsync score --format table|csv` for tabular output; `specsync generate --uncovered` and `--batch` for generating specs in bulk (#191).
- **Declarative custom validation rules** — define project-specific validation rules in config that are checked alongside built-in rules (#190).
- **Cross-repo spec content verification** — `specsync resolve --verify` fetches and validates referenced specs from remote repositories, ensuring cross-project refs point to real, valid content (#159, #195).
- **MCP resource support** — agents can browse the spec tree via 5 new MCP resources (`specsync:///specs`, `specsync:///specs/{module}`, etc.) without knowing file paths (#194).

### Fixed

- **Requirements convention docs** — clarified that requirements belong in companion `requirements.md` files, not inline in specs (#163, #193).

## [3.7.0] - 2026-04-10

### Added

- **`--no-cache` flag** — discoverable alias for `--force` that skips the hash cache (#178).
- **Cache location hint** — when specs are skipped due to caching, the path to `.specsync/hashes.json` is printed so users know where the cache lives (#178).

### Fixed

- **Absolute paths in error messages** — "No spec files found" now shows the full resolved path, making it immediately clear if you're in the wrong directory (#177).

### Changed

- **Clearer help text for spec filters** — `check` and `score` help now documents all four matching modes: module name, filename stem, relative path, and absolute path (#179).

### Closed

- **`--json` output for `score`** — already supported via the global `--json` / `--format json` flags since v3.5.0 (#172).

## [3.6.2] - 2026-04-09

### Fixed

- **`specsync diff` in PR context** — auto-detects `GITHUB_BASE_REF` in GitHub Actions so diff compares against the PR base branch instead of `HEAD` (the merge commit), which previously always reported "No files changed" (#180).

### Changed

- **Strict spec enforcement** — spec-sync now dogfoods its own `--enforcement-mode=strict` in CI, catching spec drift in the tool itself (#182).
- **100% spec file coverage** — added specs for all 62 source files (26 new spec modules), up from 58% (#183).

## [3.6.1] - 2026-04-08

### Fixed

- **`specsync new` frontmatter formatting** — `files:` and `db_tables:` fields no longer merge onto one line when source files are auto-detected (#174).
- **Empty dependency graph hint** — `specsync deps --mermaid` and `--dot` now print a helpful message when no `depends_on` relationships exist, instead of rendering only disconnected nodes (#174).

## [3.6.0] - 2026-04-08

### Added

- **Individual spec path filtering** — `specsync check` and `specsync score` now accept spec file paths or module names as positional arguments, allowing validation/scoring of specific specs instead of the entire project (#170).
- **Dependency graph visualization** — `specsync deps --mermaid` and `specsync deps --dot` output the dependency graph as Mermaid flowchart or Graphviz DOT diagrams for documentation and debugging (#152).
- **`specsync new` command** — quick-create a minimal spec with auto-detected source files and pre-populated exports. Use `--full` to also generate companion files (tasks.md, context.md, requirements.md) (#151).


## [3.5.0] - 2026-04-08

### Added

- **Stub/placeholder detection** — sections containing only "TBD", "N/A", "TODO", "Coming soon", or similar placeholders are now flagged as warnings and no longer inflate quality scores (#162).
- **Source-attributed export warnings** — undocumented export warnings now show which source file the export comes from, making them actionable in large codebases (#165).
- **Requirements companion validation** — warns when specs contain inline requirements sections (should be in `requirements.md`) and when companion files are missing (#163).
- **Score diagnostics** — `specsync score` now shows per-category breakdowns (completeness, structure, cross-references) with actionable improvement suggestions (#167).

### Fixed

- **Header matching flexibility** — fuzzy matching for common header variations like "Public API" → "Exports", "Tech Stack" → "Dependencies", reducing false negatives (#166).
- **Frontmatter parser edge cases** — correctly handles tabs, trailing whitespace, and inline YAML comments in spec frontmatter (#161).
- **`--fix` header renaming** — near-miss headers are now renamed in-place instead of duplicating the section (#164).

## [3.4.1] - 2026-04-07

### Fixed

- Added 6 missing `depends_on` entries to CLI and validator specs, resolving all `specsync deps` warnings.

## [3.4.0] - 2026-04-07

### Added

- **`specsync scaffold` command** — enhanced module scaffolding with auto-detected source files, custom template directories, and automatic registry registration (#138).
- **`specsync deps` command** — cross-module dependency graph validation detecting cycles, missing deps, and undeclared imports (#139).
- **`specsync comment` command** — post spec-sync check summaries as actionable PR comments with spec links, or print for piping (#140).
- **`specsync changelog` command** — generate changelogs of spec changes between two git refs (#141).
- **`specsync report` command** — per-module coverage report with stale and incomplete detection.
- **Graduated enforcement mode** — new `--enforcement-mode` flag with three levels: `warn` (default), `enforce-new` (errors only for new specs), and `strict` (all warnings are errors) (#134).
- **External importers** — `specsync import` supports GitHub Issues, Jira, and Confluence as spec sources (#123).
- **Interactive wizard** — `specsync wizard` for step-by-step guided spec creation (#122).
- **167+ new unit tests** across config, parser, validator, generator, and export modules.
- **100% spec coverage** — resolved 9 undocumented export warnings and added 3 missing specs.
- **Community scaffolding** — CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue/PR templates.
- **Standalone workflow guide** and onboarding documentation.

## [3.1.0] - 2026-03-30

### Added

- **`requirements.md` companion file** — a new per-module companion file scaffolded alongside `tasks.md` and `context.md` by `specsync generate` and `specsync add-spec`. The template includes User Stories, Acceptance Criteria, Constraints, and Out of Scope sections. This keeps the spec focused as a technical contract (authored by Dev/Architect) while giving Product/Design their own space for user stories and acceptance criteria.
- **AGENTS.md hook target** — `specsync hooks install --agents` installs spec-sync instructions into `AGENTS.md`, the emerging standard for multi-agent instruction files.

## [3.0.0] - 2026-03-30

### Added

- **VS Code extension** — first-class editor integration for SpecSync, published on the VS Code Marketplace as `corvidlabs.specsync`.
  - **Inline diagnostics** — errors and warnings from `specsync check --json` mapped directly to spec files with proper severity levels.
  - **CodeLens quality scores** — spec quality scores (0–100 with letter grades) displayed inline above spec files via `specsync score`.
  - **Coverage webview** — rich HTML report showing file and LOC coverage with VS Code theme-aware styling.
  - **Scoring webview** — detailed quality breakdown per spec with improvement suggestions.
  - **Five commands** — Validate Specs, Show Coverage, Score Quality, Generate Missing Specs, Initialize Config — all accessible from the Command Palette.
  - **Status bar indicator** — persistent status bar item showing pass/fail/error/syncing state with color coding.
  - **Validate-on-save** — debounced (500ms) automatic validation when spec or source files are saved.
  - **Configurable settings** — `specsync.binaryPath`, `specsync.validateOnSave`, `specsync.showInlineScores`.
  - Activates automatically in workspaces containing `specsync.json`, `.specsync.toml`, or a `specs/` directory.

### Breaking Changes

- Major version bump to v3. GitHub Action users should update to `CorvidLabs/spec-sync@v3`.

## [2.5.0] - 2026-03-30

### Added

- **Schema column validation** — SpecSync now parses SQL migrations (CREATE TABLE, ALTER TABLE ADD COLUMN) and validates documented columns in spec `### Schema` sections against the actual database schema. Catches phantom columns (documented but missing from schema), undocumented columns (in schema but not in spec), and column type mismatches. Opt-in via `schema_dir` in `specsync.json`.
- **Destructive DDL support** — migration parser correctly handles DROP TABLE, ALTER TABLE DROP COLUMN, ALTER TABLE RENAME TO, and ALTER TABLE RENAME COLUMN, ensuring the schema map accurately reflects state after all migrations replay in order.
- **Multi-language migration files** — schema extraction now supports 16 file types (SQL, TypeScript, JavaScript, Python, Ruby, Go, Rust, PHP, Swift, Kotlin, Java, C#, Dart, and more), not just `.sql`.
- **PHP language support** — full export extraction for PHP: classes, interfaces, traits, enums, public functions/constants, with visibility filtering and magic method exclusion.
- **Ruby language support** — full export extraction for Ruby: classes, modules, public methods with visibility toggle tracking, `attr_accessor`/`attr_reader`/`attr_writer`, constants, and `=begin/=end` comment handling.
- Expanded export parser test coverage for Go, Python, Java, C#, and Dart.
- Achieved 100% spec coverage across all modules.

## [2.4.0] - 2026-03-28

### Changed

- **Export validation uses allowlist** — only `### Exported ...` subsections under `## Public API` now trigger export validation. Non-export subsections (`### API Endpoints`, `### Route Handlers`, `### Component API`, `### Configuration`, etc.) are treated as informational and skipped. This fixes false errors when specs document private route handlers, component signals, service methods, or infrastructure concepts alongside validated exports (#60).

## [2.3.3] - 2026-03-28

### Documentation

- **Companion files populated** — all 28 companion files (`context.md` and `tasks.md`) across 14 modules now contain real content: architectural decisions, key files, implementation status, open tasks, known gaps, and completed work (#58).

## [2.3.2] - 2026-03-28

### Fixed

- **`action.yml` YAML parse fix** — quoted `${{ github.token }}` default value to prevent YAML stream parse errors when external repos use the action (#56).
- **`spec:check` in CI** — added spec validation to the CI pipeline so spec drift is caught automatically (#54).

### Added

- **`manifest.spec.md`** — spec for the manifest module, achieving **100% file coverage** across all 23 source files (#55).
- **Config spec update** — added `manifest` to config's `depends_on` for accurate cross-module references.

## [2.3.1] - 2026-03-28

### Added

- **`specsync-registry.toml`** — published module registry for cross-project spec resolution. Other projects can now verify refs to `CorvidLabs/spec-sync@<module>` via `resolve --remote`.

### Documentation

- **New docs page: Cross-Project References** — dedicated guide covering `owner/repo@module` syntax, registry publishing, remote verification, and CI usage.
- **CLI Reference** — added missing commands: `add-spec`, `init-registry`, `resolve`, `hooks`. Added `--format` flag documentation.
- **Spec Format** — documented cross-project ref syntax in `depends_on` field.
- **Quick Start** — added `add-spec`, `resolve`, `init-registry`, and `hooks` commands.

## [2.3.0] - 2026-03-28

### Added

- **`--format markdown` output** — `check` and `diff` commands now accept `--format markdown` to produce clean, human-readable Markdown tables instead of plain text or JSON. Useful for pasting into PRs, docs, or chat.
- **SHA256 release checksums** — release workflow now generates and publishes SHA256 checksums for all release binaries, improving supply chain verification.

### Changed

- Rolled up all v2.2.1 changes (manifest-aware modules, export granularity, language templates, robustness fixes) into this release.

## [2.2.1] - 2026-03-25 (unreleased — rolled into 2.3.0)

### Added

- **Manifest-aware module detection** — parses `Package.swift`, `Cargo.toml`, `build.gradle.kts`, `package.json`, `pubspec.yaml`, `go.mod`, and `pyproject.toml` to auto-discover targets and source paths instead of just scanning directories.
- **Export granularity control** — `"exportLevel": "type"` in `specsync.json` limits exports to top-level type declarations (class/struct/enum/protocol) instead of listing every member.
- **Configurable module definitions** — `"modules"` section in `specsync.json` lets you define module groupings with explicit file lists.
- **Language-specific spec templates** — `generate` and `--fix` produce Swift, Rust, Kotlin/Java, Go, and Python templates with appropriate section headers and table columns.
- **AI context boundary awareness** — generation prompt instructs the provider to only document symbols from the module's own files, not imported dependencies.

### Fixed

- **Test file detection** — expanded Swift patterns (Spec, Mock, Stub, Fake), added Kotlin/Java/C# patterns, and detect well-known test directories (`Tests/`, `__tests__/`, `spec/`, `mocks/`).
- **Check command no longer hangs on empty specs** — returns clean JSON/exit 0 when `--fix` is used with no spec files.
- **Exit code 101 panic → friendly error** — wraps main in `catch_unwind`, converts panics to actionable error messages with bug report link.

## [2.2.0] - 2026-03-25

### Added

- **`--fix` flag for `check` command** — automatically adds undocumented exports as stub rows in the spec's Public API table. Creates a `## Public API` section if one doesn't exist. Works with `--json` for structured output of applied fixes. Turns spec maintenance from manual bookkeeping into a one-command operation.
- **`diff` command** — compares current code exports against a git ref (default: `HEAD`) to show what's been added or removed since a given commit. Human-readable by default, `--json` for structured output. Essential for code review and CI drift detection.
- **Wildcard re-export resolution** — TypeScript/JS barrel files using `export * from './module'` now have their re-exported symbols resolved and validated. Namespace re-exports (`export * as Ns from`) are detected as a single namespace export. Resolution is depth-limited to one level to prevent infinite recursion.

### Changed

- Spec quality scoring now accounts for `--fix` generated stubs (scored lower than hand-written descriptions).
- Expanded integration test suite with 12 new tests covering `--fix`, `diff`, and wildcard re-exports (74 total integration tests, 131 total).
- Updated `cli.spec.md` and `exports.spec.md` to 100% coverage for all new features.

## [2.1.1] - 2026-03-25

### Fixed

- **Rust export extractor** — strip raw strings, char literals with `"`, and multi-line string literals before scanning for `pub` declarations. Fixes false positives from test data inside `r#"..."#` blocks, and false negatives where `'"'` char literals confused the string regex into consuming subsequent source code.
- **CLI spec** — added spec coverage for `main.rs` (CLI entry point).
- **Exports spec** — expanded to 100% file coverage across all language extractors.

## [2.1.0] - 2026-03-24

### Added

- **`specsync hooks` command** — manage agent instruction files and git hooks for spec awareness. Supports Claude Code (`CLAUDE.md`), Cursor (`.cursor/rules`), GitHub Copilot (`.github/copilot-instructions.md`), pre-commit hooks, and Claude Code hooks. Subcommands: `install`, `uninstall`, `status`.

### Security

- Updated `rustls-webpki` from 0.103.9 → 0.103.10 to fix RUSTSEC-2025-0016 (CRL Distribution Point matching logic).

### Fixed

- Spec scoring now distinguishes placeholder TODOs from descriptive references (#37).

## [2.0.0] - 2026-03-20

### Breaking Changes

- **`--ai` flag removed** — replaced by `--provider auto|claude|openai|ollama`. Use `specsync generate --provider auto` for auto-detection, or `--provider claude` for a specific provider. Plain `specsync generate` remains template-only.

### Added

- **Cross-project spec references** — specs can now reference modules in other repos via `cross_project_refs` in config. Validated locally with `specsync check`, verified remotely with `specsync resolve --remote`.
- **Companion files** — associate non-code files (migrations, configs, protos) with spec modules via `companion_files` config.
- **Spec registry** — `specsync registry` reads `specsync-registry.toml` to list and discover specs across a project.
- **`specsync resolve`** — new command to resolve cross-project references. `--remote` flag opt-in fetches registry files from GitHub repos.
- **Project scope definition** — `SCOPE.md` explicitly defines what spec-sync does and doesn't do.

### Changed

- Unified AI provider selection under `--provider` flag with auto-detection support.
- Remote ref verification groups HTTP requests by repo to minimize fetches.
- Updated all docs, examples, and tests for the new CLI surface.

## [1.3.0] - 2026-03-19

### Added

- **MCP server mode** — run `specsync mcp` to expose spec-sync as a Model Context Protocol server, enabling any AI agent (Claude Code, Cursor, Windsurf, etc.) to validate specs, check coverage, and generate specs via tool calls.
- **Direct API support** for Anthropic and OpenAI — `specsync generate --provider anthropic|openai` can call Claude or GPT APIs directly, no CLI wrapper needed. Set `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`.
- **Auto-detect source directories** — spec-sync now automatically discovers `src/`, `lib/`, `app/`, and other common source directories, so it works out-of-the-box on any project without manual config.
- **Spec quality scoring** — `specsync score` rates spec files on completeness, API coverage, section depth, and staleness, outputting a 0–100 quality score with actionable improvement suggestions.
- **TOML configuration** — `specsync.toml` is now supported alongside `specsync.json`. See `examples/specsync.toml`.
- **VS Code extension scaffold** — `vscode-extension/` directory with diagnostics, commands, and CodeLens integration (ready for Marketplace packaging).
- **Actionable error messages** — all errors and warnings now include fix suggestions.
- Expanded integration test suite (+884 lines).

### Fixed

- Resolved clippy and fmt CI failures on main (#29).

## [1.2.0] - 2026-03-19

### Added

- **`specsync generate --ai`** — AI-powered spec generation. Reads source code, sends it to an LLM, and generates specs with real content (Purpose, Public API tables, Invariants, Error Cases) instead of template stubs. Configurable via `aiCommand` and `aiTimeout` in `specsync.json`, or `SPECSYNC_AI_COMMAND` env var. Defaults to Claude CLI, works with any LLM that reads stdin and writes stdout.
- **LOC coverage tracking** — `specsync coverage` now reports lines-of-code coverage alongside file coverage. JSON output includes `loc_coverage`, `loc_covered`, `loc_total`, and `uncovered_files` with per-file LOC counts sorted by size.
- **Flat file module detection** — `generate` and `coverage` now detect single-file modules (e.g., `src/config.rs`) in addition to subdirectory-based modules.
- `aiCommand` and `aiTimeout` config options in `specsync.json`.

### Changed

- Rewrote README for density — every line carries new information, no filler.
- Documented `generate --ai` workflow, AI command configuration, and LOC coverage in README and docs site.
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

[4.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v4.0.0
[3.8.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.8.0
[3.7.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.7.0
[3.6.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.6.2
[3.6.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.6.1
[3.6.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.6.0
[3.5.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.5.0
[3.4.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.4.1
[3.4.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.4.0
[3.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.1.0
[3.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.0.0
[2.5.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.5.0
[2.4.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.4.0
[2.3.3]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.3
[2.3.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.2
[2.3.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.1
[2.3.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.0
[2.2.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.2.1
[2.2.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.2.0
[2.1.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.1.1
[2.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.1.0
[2.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.0.0
[1.3.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.3.0
[1.2.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.2.0
[1.1.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.2
[1.1.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.1
[1.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.0
[1.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.0.0
