---
module: types
version: 11
status: stable
files:
  - src/types.rs
db_tables: []
tracks: [118]
depends_on: []
---

# Types

## Purpose

Core deterministic data structures and enums shared across the codebase: configuration, validation, coverage, language detection, lifecycle, and registry entries.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `Language` | Detected source language for export extraction: TypeScript, Rust, Go, Python, Swift, Kotlin, Java, CSharp, Dart, Php, Ruby, Yaml, C, Cpp, Scala, Crystal, Nim, Erlang, Elixir, Perl, Lisp, Haskell, Lua, R, OCaml, Groovy, FSharp, Clojure, D, ObjectiveC, Bash, PowerShell, Vala |
| `OutputFormat` | CLI output format: Text (colored terminal, default), Json (machine-readable), Markdown (PR comments / agent consumption) |
| `ExportLevel` | Export extraction granularity: Type (top-level declarations only) or Member (all public symbols, default) |
| `SpecStatus` | Spec lifecycle status: draft, review, active, stable, deprecated, archived. Parsed from frontmatter `status` field |
| `EnforcementMode` | Graduated enforcement level: Warn (always exit 0), EnforceNew (exit 1 for unspecced files), Strict (exit 1 on any error) |
| `CustomRuleType` | Type of a declarative custom validation rule: RequireSection, MinWordCount, RequirePattern, ForbidPattern |
| `RuleSeverity` | Severity level for custom rules: Error, Warning (default), Info |
| `ParseMode` | Export parsing strategy: Regex (default, all languages) or Ast (tree-sitter, supports TypeScript/Python/Rust/C/C++/Scala/Erlang/Elixir/Perl/Lisp with regex fallback) |

### Exported Structs

| Type | Description |
|------|-------------|
| `Frontmatter` | YAML frontmatter parsed from a spec file (module, version, status, files, db_tables, depends_on, implements, tracks, agent_policy, lifecycle_log) |
| `ValidationResult` | Result of validating a single spec — errors, warnings, fixes, export summary, and the spec's parsed lifecycle status (so reporters can surface status-based skips) |
| `CoverageReport` | File and LOC coverage metrics for the project |
| `SpecSyncConfig` | User-provided configuration loaded from specsync.json or .specsync.toml |
| `RegistryEntry` | Registry entry mapping module names to spec file paths for cross-project resolution |
| `ModuleDefinition` | User-defined module grouping in specsync.json with files and depends_on lists |
| `ValidationRules` | Custom validation rules configured in specsync.json (required_sections, max_staleness_days, etc.) |
| `GitHubConfig` | GitHub integration config — `repo: Option<String>`, `labels: Vec<String>`, `create_on_drift: bool` |
| `CustomRule` | A declarative custom validation rule defined in specsync.json — name, type, section, pattern, min_words, severity, message, applies_to filter |
| `RuleFilter` | Filter to restrict which specs a custom rule applies to — optional status and module regex match |
| `LifecycleConfig` | Lifecycle configuration for transition guards and history tracking (guards map, track_history flag) |
| `TransitionGuard` | A transition guard — min_score, require_sections, no_stale, stale_threshold, message |
| `CompanionConfig` | Configuration for companion file generation — controls opt-in companions like design.md |

### Exported ValidationResult Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `new` | `spec_path: String` | `Self` | Create a new empty validation result |

### Exported Frontmatter Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parsed_status` | `&self` | `Option<SpecStatus>` | Parse the Frontmatter status field into a typed SpecStatus enum |

### Exported SpecStatus Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `as_str` | `&self` | `&str` | String representation of the status |
| `from_str_loose` | `s: &str` | `Option<Self>` | Parse status string into SpecStatus enum (case-insensitive) |
| `all` | — | `&[Self]` | Returns all status variants in lifecycle order |
| `ordinal` | `&self` | `usize` | Numeric position in lifecycle order (0=draft, 5=archived) |
| `next` | `&self` | `Option<Self>` | Next status in linear lifecycle (draft→review→active→stable→deprecated→archived), None at archived |
| `prev` | `&self` | `Option<Self>` | Previous status in linear lifecycle (archived→deprecated→stable→active→review→draft), None at draft |
| `valid_transitions` | `&self` | `Vec<Self>` | All valid target statuses from current (next, prev, deprecated) |
| `can_transition_to` | `&self, target: &Self` | `bool` | Whether transitioning to `target` is valid |

### Exported Language Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `from_extension` | `ext: &str` | `Option<Self>` | Detect language from file extension |
| `extensions` | `&self` | `&[&str]` | Default source file extensions for this language |
| `test_patterns` | `&self` | `&[&str]` | File patterns to exclude (test files) |

## Invariants

1. Shared types contain no inference provider or credential-bearing configuration surface
2. `Language::from_extension` returns `None` for unsupported extensions — never panics
3. `SpecSyncConfig::default()` always provides sensible deterministic defaults
4. `ValidationResult::new` initializes with empty error/warning/fix vectors and `status: None`

## Behavioral Examples

### Scenario: Detect language from file extension

- **Given** a file with extension "tsx"
- **When** `Language::from_extension("tsx")` is called
- **Then** returns `Some(Language::TypeScript)`

### Scenario: Detect Ruby from file extension

- **Given** a file with extension "rb"
- **When** `Language::from_extension("rb")` is called
- **Then** returns `Some(Language::Ruby)`

### Scenario: Unknown file extension

- **Given** a file with extension "haskell"
- **When** `Language::from_extension("haskell")` is called
- **Then** returns `None`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Unsupported file extension | `Language::from_extension` returns `None` |
| Invalid JSON config | `SpecSyncConfig` deserialization fails at the caller level |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| serde | `Deserialize` derive for configuration and frontmatter types |

### Consumed By

| Module | What is used |
|--------|-------------|
| config | `SpecSyncConfig` |
| parser | `Frontmatter` |
| validator | `CoverageReport`, `ValidationResult`, `SpecSyncConfig`, `CustomRuleType`, `RuleSeverity`, `Frontmatter` |
| generator | `CoverageReport`, `SpecSyncConfig` |
| scoring | `SpecSyncConfig` |
| exports | `Language` |
| mcp | `SpecSyncConfig` |
| registry | `RegistryEntry` |
| main | `SpecSyncConfig`, `Frontmatter` |
| github | `GitHubConfig` |
| hash_cache | `Frontmatter` |
| view | `Frontmatter` |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-11 | v2: `ValidationResult` carries the spec's parsed lifecycle status for draft-skip reporting |
| 2026-03-25 | Initial spec |
| 2026-03-28 | Document OutputFormat, ExportLevel, ModuleDefinition |
| 2026-04-06 | Add Frontmatter implements/tracks/agent_policy fields, ValidationRules, GitHubConfig structs |
| 2026-04-07 | Document EnforcementMode enum |
| 2026-04-10 | Document CustomRule, CustomRuleType, RuleSeverity, RuleFilter for declarative custom validation rules |
| 2026-04-11 | Document SpecStatus lifecycle methods (all, ordinal, next, prev, valid_transitions, can_transition_to) |
| 2026-04-11 | Move parsed_status to Frontmatter section; fix next/prev descriptions to include deprecated/archived |
| 2026-04-12 | Document CompanionConfig struct for opt-in companion file settings |
| 2026-06-07 | Remove `AiProvider::default_model` / `default_base_url` — the `corvid-ai` crate now owns the API endpoint registry and default models |
| 2026-06-07 | Add `OpenRouter`; reclassify `Ollama` as an API provider (HTTP via corvid-ai, `OLLAMA_API_KEY`); `detection_order` is now API-only; deprecate the `claude`/`copilot` CLI providers |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-14 | CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo: Count mjs and cjs files as default TypeScript sources so mapped and uncovered module files contribute to strict file and LOC coverage denominators |
| 2026-07-14 | CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str: Allow draft specs to declare planned missing source mappings without failing strict validation while preserving path safety ownership enforcement exact coverage and complete notice contracts |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-12 | CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric: Make drift gate by default: flip the default enforcement mode from warn to strict so validation errors exit non-zero without an explicit flag |
| 2026-08-13 | CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document: Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally |
| 2026-08-13 | CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di: A symlink under a source directory must be skipped and disclosed, never abort discovery |
| 2026-08-14 | CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su: A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults |
| 2026-08-14 | CHG-0118-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su: A config file that exists but cannot be loaded must refuse to run, not report success over built-in defaults |
