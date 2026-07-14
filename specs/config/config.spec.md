---
module: config
version: 7
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

Loads canonical project configuration from `.specsync/config.toml`, with compatibility fallbacks for `.specsync/config.json`, `.specsync.toml`, and `specsync.json`, then auto-detects source directories from supported language and default static HTML, HTM, and CSS files when configuration does not provide them.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `load_config` | `root: &Path` | `SpecSyncConfig` | Load configuration in canonical-to-legacy precedence order, falling back to defaults with auto-detected source directories |
| `load_config_from_path` | `config_path: &Path, root: &Path` | `SpecSyncConfig` | Load config from a specific file path (JSON or TOML based on extension), used by migration |
| `detect_source_dirs` | `root: &Path` | `Vec<String>` | Auto-detect source directories by scanning for supported language files up to 3 levels deep |
| `default_schema_pattern` | — | `&'static str` | Returns the default regex for SQL CREATE TABLE extraction |
| `discover_manifest_modules` | `root: &Path` | `ManifestDiscovery` | Discover modules from manifest files (Package.swift, Cargo.toml, etc.) |
| `is_legacy_layout` | `root: &Path` | `bool` | Detect whether a project uses a legacy 3.x layout (root-level config files without `.specsync/version` stamp) |
| `config_to_toml` | `config: &SpecSyncConfig` | `String` | Serialize a `SpecSyncConfig` to the current canonical `.specsync/config.toml` format |
| `config_to_toml_lossy_fields` | `config: &SpecSyncConfig` | `Vec<&'static str>` | List config fields `config_to_toml` cannot represent (e.g. `customRules`), so `migrate` can refuse rather than silently drop them |
| `read_config_file` | `path: &Path` | `Option<String>` | Read a config file, dropping a leading UTF-8 BOM (lossless) so it does not attach to the first TOML key or break JSON parsing; shared by the loaders and `migrate` so config reads handle a BOM consistently. `None` if unreadable |

## Invariants

1. Config file search order is `.specsync/config.toml`, `.specsync/config.json`, `.specsync.toml`, `specsync.json`, then defaults
2. When no config file exists, source directories are auto-detected from the project root
3. When a config file exists but omits canonical `source_dirs` or legacy `sourceDirs`, source dirs are still auto-detected
4. 46 common build/cache directories are always excluded from source detection (node_modules, target, .git, dist, etc.)
5. `detect_source_dirs` falls back to `["src"]` if no source files are found
6. Root-level source files (no subdirectories) produce `["."]` as source dirs
7. TOML parsing is zero-dependency — uses line-by-line string parsing, not a TOML library
8. The reader accepts both TOML string kinds for scalar and array values: basic `"..."` strings (backslash escapes decoded) and literal `'...'` strings (taken verbatim, no escape processing); a `#`, `,`, `[`, or `]` appearing inside either kind is treated as content, not as a comment or array structure
9. A config file that is absent is expected — defaults apply silently. But a config file that **exists yet cannot be read** (e.g. not valid UTF-8) fails loud: a warning naming the file is printed and built-in defaults are used, rather than silently reverting to defaults (which would downgrade enforcement — strict→warn, exit 1→0 — with no signal). The same applies to the optional local override file (`config.local.toml`)
10. Retired AI key names are ignored with migration guidance; their values are never retained, serialized, printed, or executed

## Behavioral Examples

**Scenario: Load canonical TOML config**

- **Given** `.specsync/config.toml` sets `specs_dir = "docs/specs"`
- **When** `load_config(root)` is called
- **Then** it returns `specs_dir = "docs/specs"` even if a legacy root config also exists

**Scenario: Load legacy JSON compatibility config**

- **Given** no current config exists and root `specsync.json` sets `"specsDir": "docs/specs"`
- **When** `load_config(root)` is called
- **Then** it returns `specs_dir = "docs/specs"`

**Scenario: No config file**

- **Given** none of `.specsync/config.toml`, `.specsync/config.json`, `.specsync.toml`, or `specsync.json` exists
- **When** `load_config(root)` is called
- **Then** it returns defaults with auto-detected source directories

**Scenario: Auto-detect source dirs**

- **Given** a project root with `src/` and `lib/` containing supported source files
- **When** `detect_source_dirs(root)` is called
- **Then** it returns `["lib", "src"]` in deterministic order

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Config file exists but unreadable (e.g. not valid UTF-8) | Prints a warning naming the file to stderr, then falls back to `SpecSyncConfig::default()` (fail-loud, not silent) |
| Config file absent | Silently uses `SpecSyncConfig::default()` with auto-detected source dirs (expected) |
| Malformed JSON config | Prints warning to stderr, falls back to defaults |
| Empty project root | Returns `["src"]` as source dirs |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| types | `SpecSyncConfig` |
| exports | `has_extension` for checking if files have supported language extensions |

### Consumed By

| Module | What is used |
|--------|-------------|
| validator | `load_config` (indirectly via main) |
| mcp | `load_config`, `detect_source_dirs` |
| watch | `load_config` |
| main | `load_config` |
| migrate | `load_config_from_path`, `read_config_file` (config preflight + specs_dir discovery) |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | v3: keep configuration round-trip tests warning-free under current stable Clippy |
| 2026-03-25 | Initial spec |
| 2026-03-28 | Document discover_manifest_modules |
| 2026-04-06 | Document github config section, rules section, and full config file structure |
| 2026-07-03 | Document `read_config_file` (leading-BOM-tolerant config read, shared with `migrate`) |
| 2026-07-06 | Document fail-loud behavior for present-but-unreadable config (invariant 9): unreadable config/local-override files now warn loudly before falling back to defaults; clarified Error Cases table |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0027-support-extensionless-source-discovery-through-an-explicit-include-extensionless: Support extensionless source discovery through an explicit include_extensionless setting while preserving omitted and empty source_extensions defaults, with parser, scanner, strict file coverage, and LOC coverage regressions for extensionless-only and mixed projects |

## Config File Structure

Canonical TOML uses snake_case keys and sections. Legacy JSON camelCase aliases remain readable for migration. Declarative `customRules` remain legacy-JSON-only because migration refuses rather than silently dropping data it cannot serialize.

The configuration file supports the following top-level sections:

| Section | Type | Description |
|---------|------|-------------|
| `specs_dir` (`specsDir` in legacy JSON) | `String` | Directory containing spec files (default: `"specs"`) |
| `source_dirs` (`sourceDirs` in legacy JSON) | `Vec<String>` | Source directories to scan (auto-detected if omitted) |
| `source_extensions` (`sourceExtensions` in legacy JSON) | `Vec<String>` | File extensions to consider as source files |
| `exclude_patterns` (`excludePatterns` in legacy JSON) | `Vec<String>` | Glob patterns to exclude from coverage |
| `required_sections` (`requiredSections` in legacy JSON) | `Vec<String>` | Sections every spec must contain |
| `schema_pattern` (`schemaPattern` in legacy JSON) | `String` | Regex for SQL CREATE TABLE extraction |
| `github` | `GitHubConfig` | GitHub integration settings (`repo`, `labels`, `create_on_drift`) |
| `rules` | `ValidationRules` | Custom validation rules (`max_staleness_days`, etc.) |
| `modules` | `Map<String, ModuleDefinition>` | User-defined module groupings |
