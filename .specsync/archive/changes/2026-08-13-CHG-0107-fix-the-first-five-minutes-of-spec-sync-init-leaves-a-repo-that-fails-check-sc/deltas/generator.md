## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `generate_specs_for_unspecced_modules_retained` | `project, root, report, config, progress` | `Result<GenerationOutcome, String>` | Generate CLI scaffolds through one retained project-root capability |
| `generate_specs_for_unspecced_modules` | `root, report, config` | `GenerationOutcome` | Generate local template specs for all unspecced modules with progress output |
| `generate_specs_for_unspecced_modules_paths` | `root, report, config` | `GenerationOutcome` | Generate local template specs without progress output for JSON/MCP callers |
| `generate_companion_files_for_spec` | `spec_dir, module_name, design_enabled` | `()` | Generate companion files (tasks.md, context.md, requirements.md, testing.md, and design.md if enabled) alongside a spec |
| `find_files_for_module` | `root, module_name, config` | `Vec<String>` | Find source files for a module by checking config definitions, subdirectories, then flat files |
| `find_module_source_files` | `dir: &Path, config: &SpecSyncConfig, root: &Path` | `Vec<String>` | Source files beneath a module directory, honoring configured extensions and exclusions; shared with spec validation so a directory in `files:` is corrected with exactly what generation would have written |
| `find_single_source_fallback` | `root, config` | `Option<String>` | Root-relative path of the project's only non-test source file (e.g. `src/lib.rs`), or `None` when there are zero or multiple candidates — fallback for `new`/`scaffold` when no name match exists |
| `generate_spec` | `module_name, source_files, root, specs_dir` | `String` | Generate a spec from a template (custom or language-aware default) |
| `generate_spec_from_custom_template` | `template_dir, module_name, source_files, root` | `String` | Generate a spec using files from a custom template directory |
| `generate_companion_files_from_template` | `spec_dir, module_name, template_dir, design_enabled` | `()` | Generate companion files from a custom template directory with fallback to defaults; creates design.md only when `design_enabled` is true |
| `collect_exports_for_files` | `root, source_files` | `Vec<String>` | Collect exported symbols across the given source files |
| `populate_public_api_table` | `spec, exports` | `String` | Insert or refresh a Public API table from discovered export names |

**Exported Types**

| Type | Description |
|------|-------------|
| `GenerationOutcome` | Deterministic generation result: generated count and relative generated paths |

