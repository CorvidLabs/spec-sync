## ADDED

### REQUIREMENT REQ-config-013

Module configuration SHALL accept `owns` under `[modules."<name>"]` as a list of project-relative paths the module owns for the change lifecycle beyond its spec's `files:`.

Acceptance Criteria
- `owns` is parsed from canonical TOML and legacy JSON, typed as an array of strings by the checked parser, and serialized by `config_to_toml` beside `files` and `depends_on`; a module carrying only `owns` still round-trips.
- The key is not a source mapping: coverage, export extraction, and `find_files_for_module` ignore it, and an owned path is never demanded to have spec coverage.
- An omitted `owns` leaves every existing behaviour unchanged.

## MODIFIED

### SPEC SECTION Config File Structure

Canonical TOML uses snake_case keys and sections. Legacy JSON camelCase aliases remain readable for migration. Declarative `customRules` remain legacy-JSON-only because migration refuses rather than silently dropping data it cannot serialize.

The configuration file supports the following top-level sections:

| Section | Type | Description |
|---------|------|-------------|
| `specs_dir` (`specsDir` in legacy JSON) | `String` | Directory containing spec files (default: `"specs"`) |
| `source_dirs` (`sourceDirs` in legacy JSON) | `Vec<String>` | Source directories to scan (auto-detected if omitted) |
| `source_dirs_set` | `bool` | Runtime-only: whether the file stated `source_dirs` rather than having it inferred. Never serialized |
| `source_extensions` (`sourceExtensions` in legacy JSON) | `Vec<String>` | File extensions to consider as source files |
| `exclude_patterns` (`excludePatterns` in legacy JSON) | `Vec<String>` | Glob patterns to exclude from coverage |
| `required_sections` (`requiredSections` in legacy JSON) | `Vec<String>` | Sections every spec must contain |
| `require_draft_files` (`requireDraftFiles` in legacy JSON) | `bool` | Require missing `draft` mappings to exist immediately instead of reporting them as planned (default: `false`) |
| `schema_pattern` (`schemaPattern` in legacy JSON) | `String` | Regex for SQL CREATE TABLE extraction |
| `github` | `GitHubConfig` | GitHub integration settings (`repo`, `labels`, `create_on_drift`) |
| `rules` | `ValidationRules` | Custom validation rules (`max_staleness_days`, etc.) |
| `modules` | `Map<String, ModuleDefinition>` | User-defined module groupings: `files` (a source mapping), `depends_on`, and `owns` — project-relative files or directories the module owns for change acceptance beyond its spec's `files:`, read by the change lifecycle alone and never a source mapping |
