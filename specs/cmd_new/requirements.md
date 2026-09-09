---
spec: cmd_new.spec.md
---

## User Stories

- As a developer starting a new module, I want `specsync new <module>` to scaffold a spec in one step so that I don't hand-write frontmatter and boilerplate sections.
- As a developer, I want the new spec to auto-detect the module's source files (a matching `src/<module>/` directory or a `src/<module>.<ext>` file) so that the `files:` list is correct from the start.
- As a developer, I want the Public API table pre-populated with the module's exported symbols so that I only have to fill in descriptions, not discover the surface area.
- As a developer, I want `--full` to also create the companion files (tasks, context, requirements, testing — plus design when configured) so that a module's spec set is complete from creation.
- As a careful user, I want the command to refuse to overwrite an existing spec so that I never lose hand-written content.

## Acceptance Criteria

- `specsync new <module>` creates `<specs_dir>/<module>/<module>.spec.md` with frontmatter (`module`, `version: 1`, `status: draft`, `files:`, `db_tables: []`, `depends_on: []`) and the standard section skeleton (Purpose, Public API, Dependencies, Change Log).
- Source files are detected by scanning each configured `source_dirs` entry for a directory named `<module>` (recursively) and for a top-level file whose stem equals `<module>`, filtered by `source_extensions`; detected paths are relative, forward-slash, sorted, and de-duplicated.
- When no source files are found, the spec is still created with `files: []`.
- Exported symbols from the detected source files are collected via `generator::generate_spec` / `collect_exports_for_files`, de-duplicated, and rendered as Public API rows; each row carries a review prompt to document the export rather than a placeholder marker.
- `--full` invokes `generator::generate_companion_files_for_spec`, creating `tasks.md`, `context.md`, `requirements.md`, `testing.md`, and `design.md` only when `companions.design` is enabled in config.
- An existing target spec file causes exit code 1 with an error message; the command never overwrites it.

## Constraints

- Must not panic on expected error conditions — print an error and exit non-zero.
- Dates come from the shared generator (no in-module `chrono_lite_today` helper and no `chrono` dependency) and must be cross-platform.
- Path output is normalized to forward slashes so generated specs are stable across Windows and Unix.
- Honors the project's `specs_dir`, `source_dirs`, `source_extensions`, and `companions.design` config values.

## Out of Scope

- Filling in section prose, invariants, or dependency descriptions (the command scaffolds; the author writes content).
- Inferring `depends_on` from source imports (emitted as `depends_on: []`).
- Interactive prompts (see the `wizard` command) and any GUI/web interface.
- Updating the registry or running validation as part of creation.

### REQ-cmd-new-001

`specsync new` SHALL emit every heading in the project's `required_sections` (the same skeleton `add-spec` / `generate` write), not a four-section subset.

Acceptance Criteria
- A spec created by `new` contains Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, and Change Log.
- Public API rows are pre-populated from detected exports via `generator::generate_spec`.
- The private `chrono_lite_today` helper is gone; the Change Log date comes from the shared generator.
- The canonical spec body describes `generate_spec`, `validate_scaffold_module_name`, `check_case_collision`, `collect_exports_for_files`, and `generate_companion_files_for_spec` — not `chrono_lite_today`, `validate_module_name`, `get_exported_symbols`, `has_extension`, or `generate_companion_files`.

### REQ-cmd-new-002

The new command SHALL exclude recognized test files from auto-detected module sources.

Acceptance Criteria

- JavaScript-family `.test.*` and `.spec.*` files are omitted.
- Production files with configured or default source extensions remain included.


### REQ-cmd-new-003

`new` SHALL derive exports from the configured surface and parse mode.

Acceptance Criteria
- The Public API it writes contains only symbols `check` will accept.

### REQ-cmd-new-004

`specsync new` SHALL refuse a module name under the same rules as `scaffold` (`validate_scaffold_module_name` plus case-collision).

Acceptance Criteria
- Reserved names (`change`, `specs`, `con`), leading dashes, spaces, and path separators exit 1 with `invalid module name` and write nothing.
- A case-fold collision with an existing spec directory is refused before any write.

