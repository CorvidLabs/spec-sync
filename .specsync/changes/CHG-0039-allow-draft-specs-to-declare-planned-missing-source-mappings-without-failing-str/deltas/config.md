## ADDED

### REQUIREMENT REQ-config-004

Configuration SHALL provide and document a default-false `require_draft_files` option, named `requireDraftFiles` in legacy JSON, that requires every draft mapping to exist when enabled.

Acceptance Criteria

- Omitted and explicit-false values preserve planned draft mappings.
- Canonical TOML reads and emits `require_draft_files = true` without losing the value during migration.
- Legacy JSON reads `requireDraftFiles` and recognizes it as a supported key.
- The canonical configuration structure table documents both serialized names and behavior.


## MODIFIED

### SPEC SECTION Config File Structure

Canonical TOML uses snake_case keys and sections. Legacy JSON camelCase aliases remain readable for migration. Declarative `customRules` remain legacy-JSON-only because migration refuses rather than silently dropping data it cannot serialize.

The configuration file supports the following top-level sections:

| Section | Type | Description |
|---------|------|-------------|
| `specs_dir` (`specsDir` in legacy JSON) | `String` | Directory containing spec files (default: `"specs"`) |
| `source_dirs` (`sourceDirs` in legacy JSON) | `Vec<String>` | Source directories to scan (auto-detected if omitted) |
| `source_extensions` (`sourceExtensions` in legacy JSON) | `Vec<String>` | File extensions to consider as source files |
| `exclude_patterns` (`excludePatterns` in legacy JSON) | `Vec<String>` | Glob patterns to exclude from coverage |
| `required_sections` (`requiredSections` in legacy JSON) | `Vec<String>` | Sections every spec must contain |
| `require_draft_files` (`requireDraftFiles` in legacy JSON) | `bool` | Require missing `draft` mappings to exist immediately instead of reporting them as planned (default: `false`) |
| `schema_pattern` (`schemaPattern` in legacy JSON) | `String` | Regex for SQL CREATE TABLE extraction |
| `github` | `GitHubConfig` | GitHub integration settings (`repo`, `labels`, `create_on_drift`) |
| `rules` | `ValidationRules` | Custom validation rules (`max_staleness_days`, etc.) |
| `modules` | `Map<String, ModuleDefinition>` | User-defined module groupings |
