---
module: generator
version: 1
status: stable
files:
  - src/generator.rs
db_tables: []
tracks: [73]
depends_on:
  - specs/types/types.spec.md
  - specs/ai/ai.spec.md
  - specs/exports/exports.spec.md
---

# Generator

## Purpose

Scaffolds spec files and companion files (tasks.md, context.md, requirements.md, testing.md, and optionally design.md) for unspecced modules. Supports both template-based generation (using a default or custom `_template.spec.md`) and AI-powered generation that reads source code and calls an LLM to produce meaningful specs.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `generate_specs_for_unspecced_modules` | `root, report, config, provider` | `usize` | Generate specs for all unspecced modules, returning count of generated specs |
| `generate_specs_for_unspecced_modules_paths` | `root, report, config, provider` | `Vec<String>` | Generate specs for all unspecced modules, returning paths of generated files |
| `generate_companion_files_for_spec` | `spec_dir, module_name, design_enabled` | `()` | Generate companion files (tasks.md, context.md, requirements.md, testing.md, and design.md if enabled) alongside a spec |
| `find_files_for_module` | `root, module_name, config` | `Vec<String>` | Find source files for a module by checking config definitions, subdirectories, then flat files |
| `generate_spec` | `module_name, source_files, root, specs_dir` | `String` | Generate a spec from a template (custom or language-aware default) |
| `generate_spec_from_custom_template` | `template_dir, module_name, source_files, root` | `String` | Generate a spec using files from a custom template directory |
| `generate_companion_files_from_template` | `spec_dir, module_name, template_dir` | `()` | Generate companion files from a custom template directory with fallback to defaults |

## Invariants

1. Specs are never overwritten — if a `module.spec.md` already exists, it is skipped
2. Custom templates at `specs/_template.spec.md` take precedence over the built-in default
3. Template generation fills in module name, version (1), status (draft), and discovered source files
4. Module title is derived from the module name with dashes converted to title case (e.g. "api-gateway" -> "Api Gateway")
5. Companion files (tasks.md, context.md, requirements.md, testing.md, and design.md when enabled) are only created if they don't already exist
6. AI generation falls back to template on failure (with a warning to stderr)
7. Source file paths in frontmatter are relative to the project root
8. Module source files are discovered by checking subdirectory-based modules first, then flat files

## Behavioral Examples

### Scenario: Generate spec for unspecced module

- **Given** a module "auth" with source files in `src/auth/` and no existing spec
- **When** `generate_specs_for_unspecced_modules` is called
- **Then** creates `specs/auth/auth.spec.md`, `specs/auth/tasks.md`, `specs/auth/context.md`, `specs/auth/requirements.md`, and `specs/auth/testing.md`

### Scenario: Skip existing spec

- **Given** a module "auth" that already has `specs/auth/auth.spec.md`
- **When** `generate_specs_for_unspecced_modules` is called
- **Then** skips the module, returns 0

### Scenario: AI generation fallback

- **Given** an AI provider that fails with an error
- **When** generating a spec for module "auth"
- **Then** falls back to template-based generation and prints a warning

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Cannot create spec directory | Prints error to stderr, skips module |
| Cannot write spec file | Prints error to stderr, skips module |
| AI generation fails | Falls back to template, prints warning |
| No source files found for module | Skips module entirely |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| ai | `generate_spec_with_ai`, `ResolvedProvider` |
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
| 2026-03-25 | Initial spec |
| 2026-04-07 | Document find_files_for_module, generate_spec, generate_spec_from_custom_template, generate_companion_files_from_template |
| 2026-04-12 | Update companion files list to include requirements.md, testing.md, and opt-in design.md; add design_enabled parameter |
