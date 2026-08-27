## MODIFIED

### SPEC SECTION Invariants

1. Config file search order is `.specsync/config.toml`, `.specsync/config.json`, `.specsync.toml`, `specsync.json`, then defaults.
2. When no config file exists, source directories are auto-detected from the project root.
3. When a config file exists but omits canonical `source_dirs` or legacy `sourceDirs`, source dirs are still auto-detected.
   Which of the two happened is recorded on `SpecSyncConfig.source_dirs_set`, because a configured `["src"]` and the
   `["src"]` default are indistinguishable once loading is done, and coverage has to tell a stated source list from an
   inferred one to decide what a manifest it cannot parse is allowed to veto.
4. 46 common build/cache directories are always excluded from source detection.
5. `detect_source_dirs` falls back to `["src"]` if no source files are found.
6. Root-level source files produce `["."]` as source dirs.
7. TOML parsing is zero-dependency and uses line-by-line string parsing.
8. Basic and literal TOML strings preserve punctuation as content according to their string kind.
9. Present-but-unreadable config and local override files warn before built-in defaults are used; absent files apply defaults silently.
10. Retired AI key names are ignored with value-safe migration guidance and are never retained, serialized, printed, or executed.
11. Checked source-directory and manifest discovery fail before returning partial results when Gradle settings are malformed or unreadable; compatibility wrappers remain infallible for existing callers.
12. Checked retained-snapshot parsing validates real JSON/TOML syntax and known TOML field types
    before applying the established compatibility parser.
13. Capability callers may supply source-directory detection; omitted source fields consume that
    list without consulting an ambient root pathname.
14. Security-sensitive zero-config source detection begins only after the caller retains the
    project root and consumes manifest observations obtained through that capability.
15. Retained CLI discovery reads config bytes through its project capability, honors explicit
    source lists without pre-traversal, and preserves malformed legacy config warning fallback.
16. Nested configuration parents are reverified through the retained project root around the
    bounded read; a detached parent cannot become mixed-generation configuration authority.

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
| `modules` | `Map<String, ModuleDefinition>` | User-defined module groupings |
