## MODIFIED

### REQUIREMENT REQ-cmd-new-001

`specsync new` SHALL emit every heading in the project's `required_sections` (the same skeleton `add-spec` / `generate` write), not a four-section subset.

Acceptance Criteria
- A spec created by `new` contains Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, and Change Log.
- Public API rows are pre-populated from detected exports via `generator::generate_spec`.
- The private `chrono_lite_today` helper is gone; the Change Log date comes from the shared generator.
- The canonical spec body describes `generate_spec`, `validate_scaffold_module_name`, `check_case_collision`, `collect_exports_for_files`, and `generate_companion_files_for_spec` — not `chrono_lite_today`, `validate_module_name`, `get_exported_symbols`, `has_extension`, or `generate_companion_files`.

### SPEC SECTION Invariants

1. Auto-detects source files by scanning source dirs for module name matches; when nothing matches and the project has exactly one non-test source file (e.g. only `src/lib.rs`), that file is used as the module's source
2. Extracts exports to pre-populate Public API tables via `generator::generate_spec`
3. `--full` generates companion files (tasks.md, context.md, requirements.md, testing.md) via `generator::generate_companion_files_for_spec()`; design.md is included only when `companions.design` is enabled in config
4. Emits the same seven-section skeleton as `add-spec` / `generate`; Change Log dates come from the shared generator, not a private `chrono_lite_today()` helper
5. Will not overwrite existing spec
6. Module names are refused under the same rules as `scaffold` (`validate_scaffold_module_name` plus case-collision)

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec already exists | Exits 1 |
| No source files found | Creates spec with empty `files:` and prints a ⚠ explaining that the `files:` list must be filled in before `check` passes |
| Dir creation fails | Exits 1 |
| Invalid module name (path separator, `.`/`..`, absolute/drive-relative, control chars, reserved names, leading dashes, spaces) | Refused via `validate_scaffold_module_name` before any write; prints `invalid module name …` and exits 1 (no path traversal) |
| Case-fold collision with an existing spec directory | Refused via `check_case_collision` before any write |

### SPEC SECTION Dependencies

#### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| exports | `has_configured_extension`, `is_test_file` |
| generator | `generate_spec`, `collect_exports_for_files`, `generate_companion_files_for_spec`, `find_single_source_fallback` |
| commands | `validate_scaffold_module_name`, `check_case_collision` |

#### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync new` |
