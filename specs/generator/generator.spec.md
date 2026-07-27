---
module: generator
version: 8
status: stable
files:
  - src/generator.rs
db_tables: []
tracks: [73]
depends_on:
  - specs/types/types.spec.md
  - specs/exports/exports.spec.md
---

# Generator

## Purpose

Deterministically scaffolds spec files and companion files for unspecced modules using built-in or project-owned templates.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `generate_specs_for_unspecced_modules_retained` | `project, root, report, config, progress` | `Result<GenerationOutcome, String>` | Generate CLI scaffolds through one retained project-root capability |
| `generate_specs_for_unspecced_modules` | `root, report, config` | `GenerationOutcome` | Generate local template specs for all unspecced modules with progress output |
| `generate_specs_for_unspecced_modules_paths` | `root, report, config` | `GenerationOutcome` | Generate local template specs without progress output for JSON/MCP callers |
| `generate_companion_files_for_spec` | `spec_dir, module_name, design_enabled` | `()` | Generate companion files (tasks.md, context.md, requirements.md, testing.md, and design.md if enabled) alongside a spec |
| `find_files_for_module` | `root, module_name, config` | `Vec<String>` | Find source files for a module by checking config definitions, subdirectories, then flat files |
| `find_single_source_fallback` | `root, config` | `Option<String>` | Root-relative path of the project's only non-test source file (e.g. `src/lib.rs`), or `None` when there are zero or multiple candidates — fallback for `new`/`scaffold` when no name match exists |
| `generate_spec` | `module_name, source_files, root, specs_dir` | `String` | Generate a spec from a template (custom or language-aware default) |
| `generate_spec_from_custom_template` | `template_dir, module_name, source_files, root` | `String` | Generate a spec using files from a custom template directory |
| `generate_companion_files_from_template` | `spec_dir, module_name, template_dir, design_enabled` | `()` | Generate companion files from a custom template directory with fallback to defaults; creates design.md only when `design_enabled` is true |

### Exported Types

| Type | Description |
|------|-------------|
| `GenerationOutcome` | Deterministic generation result: generated count and relative generated paths |

## Invariants

- CLI generation confines template reads, directory creation, and no-overwrite publication to one
  retained project-root capability so public-root replacement cannot redirect output.

## Behavioral Examples

### Scenario: Generate spec for unspecced module

- **Given** a module "auth" with source files in `src/auth/` and no existing spec
- **When** `generate_specs_for_unspecced_modules` is called
- **Then** creates `specs/auth/auth.spec.md`, `specs/auth/tasks.md`, `specs/auth/context.md`, `specs/auth/requirements.md`, `specs/auth/testing.md`, and `specs/auth/design.md` if `companions.design` is enabled in config

### Scenario: Skip existing spec

- **Given** a module "auth" that already has `specs/auth/auth.spec.md`
- **When** `generate_specs_for_unspecced_modules` is called
- **Then** skips the module, returns an outcome with `generated == 0`

### Scenario: Design companion opt-in

- **Given** `companions.design` is enabled in config
- **When** `generate_companion_files_for_spec` is called for module "dashboard"
- **Then** creates design.md with YAML frontmatter (`spec: dashboard.spec.md`, `sources: []`) and sections for Layout, Components, Tokens, Assets

### Scenario: Design companion disabled by default

- **Given** no `companions.design` config (default: false)
- **When** `generate_companion_files_for_spec` is called
- **Then** creates tasks.md, context.md, requirements.md, testing.md but NOT design.md

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot create spec directory | Prints error to stderr, skips module |
| Cannot write spec file | Prints error to stderr, skips module |
| No source files found for module | Skips module entirely |
| Requested CLI root is replaced after coverage | Generation fails inconclusively and writes to neither the replacement nor an outside path |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| exports | `has_extension`, `is_test_file` |
| types | `CoverageReport`, `SpecSyncConfig` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `generate_specs_for_unspecced_modules`, `generate_companion_files_for_spec` |
| mcp | `generate_specs_for_unspecced_modules_paths` |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | v7 / CHG-0063: Bind CLI generation templates, directories, and no-overwrite file publication to one retained project-root capability |
| 2026-07-10 | v5: keep module-discovery test fixtures warning-free under current stable Clippy |
| 2026-03-25 | Initial spec |
| 2026-04-07 | Document find_files_for_module, generate_spec, generate_spec_from_custom_template, generate_companion_files_from_template |
| 2026-04-12 | Update companion files list to include requirements.md, testing.md, and opt-in design.md; add design_enabled parameter |
| 2026-04-13 | Fix generate_companion_files_from_template signature to include design_enabled; update scenario for conditional design.md |
| 2026-06-07 | Replace unfinished-marker built-in template content with guided starter content |
| 2026-06-11 | Return `GenerationOutcome` (count, paths, AI errors) from both generation entry points so AI failures surface with a non-zero exit |
| 2026-06-11 | Add `find_single_source_fallback` so `new`/`scaffold` auto-detect the source in single-source-file projects (e.g. a fresh cargo crate with only `src/lib.rs`) |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
