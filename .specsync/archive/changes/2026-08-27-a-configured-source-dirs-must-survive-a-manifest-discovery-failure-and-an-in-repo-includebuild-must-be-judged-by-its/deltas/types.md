## MODIFIED

### SPEC SECTION Public API

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
| `CoverageReport` | File and LOC coverage metrics for the project, plus what shaped them: files referenced but missing, links not traversed, and manifests degraded rather than propagated |
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
| `measured_file_total` | `&self` | `usize` | Files that count toward coverage: those discovered plus those a spec names but that are absent, so a broken `files:` list cannot shrink the denominator |
| `file_coverage` | `&self` | `Option<f64>` | Fraction of source files covered by a spec, or `None` when there are no files to measure |
| `file_coverage_percent` | `&self` | `Option<usize>` | `file_coverage` as a whole percent. `None` means nothing was measured, which is distinct from `Some(0)` |
| `loc_coverage` | `&self` | `Option<f64>` | Fraction of source lines covered by a spec, or `None` when there are no lines to measure |
| `loc_coverage_percent` | `&self` | `Option<usize>` | `loc_coverage` as a whole percent, `None` when unmeasured |
