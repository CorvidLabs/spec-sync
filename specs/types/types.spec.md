---
module: types
version: 1
status: stable
files:
  - src/types.rs
db_tables: []
tracks: [118]
depends_on: []
---

# Types

## Purpose

Core data structures and enums shared across the entire spec-sync codebase. Defines the configuration schema, validation results, coverage reports, AI provider presets, language detection, and registry entries.

## Public API

### Exported Enums

| Type | Description |
|------|-------------|
| `AiProvider` | Supported AI provider presets: Claude, Cursor, Copilot, Ollama, Anthropic, OpenAi, Custom |
| `Language` | Detected source language for export extraction: TypeScript, Rust, Go, Python, Swift, Kotlin, Java, CSharp, Dart, Php, Ruby |
| `OutputFormat` | CLI output format: Text (colored terminal, default), Json (machine-readable), Markdown (PR comments / agent consumption) |
| `ExportLevel` | Export extraction granularity: Type (top-level declarations only) or Member (all public symbols, default) |
| `SpecStatus` | Spec lifecycle status: draft, review, active, stable, deprecated, archived. Parsed from frontmatter `status` field |
| `EnforcementMode` | Graduated enforcement level: Warn (always exit 0), EnforceNew (exit 1 for unspecced files), Strict (exit 1 on any error) |
| `CustomRuleType` | Type of a declarative custom validation rule: RequireSection, MinWordCount, RequirePattern, ForbidPattern |
| `RuleSeverity` | Severity level for custom rules: Error, Warning (default), Info |
| `ParseMode` | Export parsing strategy: Regex (default, all languages) or Ast (tree-sitter, supports TypeScript/Python/Rust with regex fallback) |

### Exported Structs

| Type | Description |
|------|-------------|
| `Frontmatter` | YAML frontmatter parsed from a spec file (module, version, status, files, db_tables, depends_on, implements, tracks, agent_policy, lifecycle_log) |
| `ValidationResult` | Result of validating a single spec — errors, warnings, fixes, and export summary |
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

### Exported AiProvider Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `default_command` | `&self` | `Option<&'static str>` | CLI command string for this provider (None for API-only) |
| `binary_name` | `&self` | `&'static str` | Binary name to check availability (empty for API providers) |
| `is_api_provider` | `&self` | `bool` | Whether this provider uses direct API calls |
| `api_key_env_var` | `&self` | `Option<&'static str>` | Environment variable name for the API key |
| `default_model` | `&self` | `Option<&'static str>` | Default model name for API providers |
| `from_str_loose` | `s: &str` | `Option<Self>` | Parse provider name from string (case-insensitive, aliases supported) |
| `detection_order` | — | `&'static [AiProvider]` | All auto-detectable providers in preference order |

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
| `default_base_url` | <!-- TODO: describe --> |

## Invariants

1. `AiProvider::from_str_loose` is case-insensitive and accepts common aliases (e.g. "gh-copilot" -> Copilot)
2. `AiProvider::detection_order` returns CLI providers before API providers
3. `Language::from_extension` returns `None` for unsupported extensions — never panics
4. `SpecSyncConfig::default()` always provides sensible defaults (specs_dir="specs", source_dirs=["src"], 7 required sections)
5. `ValidationResult::new` initializes with empty error/warning/fix vectors

## Behavioral Examples

### Scenario: Parse AI provider from string

- **Given** the string "anthropic-api"
- **When** `AiProvider::from_str_loose("anthropic-api")` is called
- **Then** returns `Some(AiProvider::Anthropic)`

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
| Unknown provider string | `AiProvider::from_str_loose` returns `None` |
| Unsupported file extension | `Language::from_extension` returns `None` |
| Invalid JSON config | `SpecSyncConfig` deserialization fails at the caller level |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| serde | `Deserialize` derive for `SpecSyncConfig` and `AiProvider` |

### Consumed By

| Module | What is used |
|--------|-------------|
| config | `SpecSyncConfig`, `AiProvider` |
| parser | `Frontmatter` |
| validator | `CoverageReport`, `ValidationResult`, `SpecSyncConfig`, `CustomRuleType`, `RuleSeverity`, `Frontmatter` |
| generator | `CoverageReport`, `SpecSyncConfig` |
| ai | `AiProvider`, `SpecSyncConfig` |
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
| 2026-03-25 | Initial spec |
| 2026-03-28 | Document OutputFormat, ExportLevel, ModuleDefinition |
| 2026-04-06 | Add Frontmatter implements/tracks/agent_policy fields, ValidationRules, GitHubConfig structs |
| 2026-04-07 | Document EnforcementMode enum |
| 2026-04-10 | Document CustomRule, CustomRuleType, RuleSeverity, RuleFilter for declarative custom validation rules |
| 2026-04-11 | Document SpecStatus lifecycle methods (all, ordinal, next, prev, valid_transitions, can_transition_to) |
| 2026-04-11 | Move parsed_status to Frontmatter section; fix next/prev descriptions to include deprecated/archived |
