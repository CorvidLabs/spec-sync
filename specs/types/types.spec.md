---
module: types
version: 1
status: stable
files:
  - src/types.rs
db_tables: []
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
| `SpecStatus` | Spec lifecycle status: draft, stable, deprecated. Parsed from frontmatter `status` field |

### Exported Structs

| Type | Description |
|------|-------------|
| `Frontmatter` | YAML frontmatter parsed from a spec file (module, version, status, files, db_tables, depends_on, implements, tracks, agent_policy) |
| `ValidationResult` | Result of validating a single spec — errors, warnings, fixes, and export summary |
| `CoverageReport` | File and LOC coverage metrics for the project |
| `SpecSyncConfig` | User-provided configuration loaded from specsync.json or .specsync.toml |
| `RegistryEntry` | Registry entry mapping module names to spec file paths for cross-project resolution |
| `ModuleDefinition` | User-defined module grouping in specsync.json with files and depends_on lists |
| `ValidationRules` | Custom validation rules configured in specsync.json (required_sections, max_staleness_days, etc.) |
| `GitHubConfig` | GitHub integration config — `repo: Option<String>`, `labels: Vec<String>`, `create_on_drift: bool` |

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

### Exported SpecStatus Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `as_str` | `&self` | `&str` | String representation of the status |
| `parsed_status` | `s: &str` | `Option<Self>` | Parse status string into SpecStatus enum |

### Exported Language Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `from_extension` | `ext: &str` | `Option<Self>` | Detect language from file extension |
| `extensions` | `&self` | `&[&str]` | Default source file extensions for this language |
| `test_patterns` | `&self` | `&[&str]` | File patterns to exclude (test files) |

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
| validator | `CoverageReport`, `ValidationResult`, `SpecSyncConfig` |
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
