---
module: config
version: 1
status: stable
files:
  - src/config.rs
db_tables: []
tracks: [31]
depends_on:
  - specs/types/types.spec.md
  - specs/exports/exports.spec.md
  - specs/manifest/manifest.spec.md
---

# Config

## Purpose

Loads project configuration from `specsync.json` or `.specsync.toml`, with fallback to auto-detected defaults. Auto-detects source directories by scanning for files with supported language extensions.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_config` | `root: &Path` | `SpecSyncConfig` | Load config from specsync.json or .specsync.toml, falling back to defaults with auto-detected source dirs |
| `detect_source_dirs` | `root: &Path` | `Vec<String>` | Auto-detect source directories by scanning for supported language files up to 3 levels deep |
| `default_schema_pattern` | — | `&'static str` | Returns the default regex for SQL CREATE TABLE extraction |
| `discover_manifest_modules` | `root: &Path` | `ManifestDiscovery` | Discover modules from manifest files (Package.swift, Cargo.toml, etc.) |
| `is_legacy_layout` | `root: &Path` | `bool` | Detect whether a project uses a legacy 3.x layout (root-level config files without `.specsync/version` stamp) |
| `config_to_toml` | `config: &SpecSyncConfig` | `String` | Serialize a `SpecSyncConfig` to TOML format string for v4.0.0 config files |

### Config File Structure

The configuration file supports the following top-level sections:

| Section | Type | Description |
|---------|------|-------------|
| `specsDir` | `String` | Directory containing spec files (default: `"specs"`) |
| `sourceDirs` | `Vec<String>` | Source directories to scan (auto-detected if omitted) |
| `sourceExtensions` | `Vec<String>` | File extensions to consider as source files |
| `excludePatterns` | `Vec<String>` | Glob patterns to exclude from coverage |
| `requiredSections` | `Vec<String>` | Sections every spec must contain |
| `schemaPattern` | `String` | Regex for SQL CREATE TABLE extraction |
| `github` | `GitHubConfig` | GitHub integration settings (`repo`, `labels`, `create_on_drift`) |
| `rules` | `ValidationRules` | Custom validation rules (`max_staleness_days`, etc.) |
| `modules` | `Map<String, ModuleDefinition>` | User-defined module groupings |

## Invariants

1. Config file search order: `specsync.json` first, then `.specsync.toml`, then defaults
2. When no config file exists, source directories are auto-detected from the project root
3. When a config file exists but omits `sourceDirs`, source dirs are still auto-detected
4. 46 common build/cache directories are always excluded from source detection (node_modules, target, .git, dist, etc.)
5. `detect_source_dirs` falls back to `["src"]` if no source files are found
6. Root-level source files (no subdirectories) produce `["."]` as source dirs
7. TOML parsing is zero-dependency — uses line-by-line string parsing, not a TOML library

## Behavioral Examples

### Scenario: Load JSON config

- **Given** a `specsync.json` exists at project root with `"specsDir": "docs/specs"`
- **When** `load_config(root)` is called
- **Then** returns a config with `specs_dir = "docs/specs"`

### Scenario: No config file

- **Given** no `specsync.json` or `.specsync.toml` exists
- **When** `load_config(root)` is called
- **Then** returns default config with auto-detected source dirs

### Scenario: Auto-detect source dirs

- **Given** a project root with `src/` and `lib/` containing `.rs` files
- **When** `detect_source_dirs(root)` is called
- **Then** returns `["lib", "src"]` (sorted alphabetically)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Config file unreadable | Falls back to `SpecSyncConfig::default()` |
| Malformed JSON config | Prints warning to stderr, falls back to defaults |
| Empty project root | Returns `["src"]` as source dirs |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `SpecSyncConfig`, `AiProvider` |
| exports | `has_extension` for checking if files have supported language extensions |

### Consumed By

| Module | What is used |
|--------|-------------|
| validator | `load_config` (indirectly via main) |
| mcp | `load_config`, `detect_source_dirs` |
| watch | `load_config` |
| main | `load_config` |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
| 2026-03-28 | Document discover_manifest_modules |
| 2026-04-06 | Document github config section, rules section, and full config file structure |
