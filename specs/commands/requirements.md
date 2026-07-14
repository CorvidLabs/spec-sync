---
spec: commands.spec.md
---

## User Stories

- As a subcommand author, I want shared config-load, discovery, filtering, and validation helpers so that I don't re-implement boilerplate in every `commands::*` module
- As a CLI user, I want `--exclude-status` / `--only-status` and positional spec filters honored consistently across commands so that scoping a run behaves the same everywhere
- As a CI operator, I want exit codes that reflect enforcement mode and coverage thresholds so that pipeline pass/fail is predictable and actionable
- As a maintainer, I want exit-code logic separated from process exit so that the decision is unit-testable without spawning a process
- As a team using GitHub, I want validation errors turned into one issue per drifted spec so that drift gets tracked without flooding the tracker

## Acceptance Criteria

- `load_and_discover` loads config, discovers spec files under the configured specs dir, excludes `_`-prefixed files, and exits 0 with a "run `specsync generate`" hint when empty and `allow_empty` is false
- `filter_specs` matches a filter against exact path, project-relative path, filename stem, or module name (stem minus `.spec`); empty filters return all specs; unmatched filters print a yellow warning
- `filter_by_status` supports exclude-list and only-list modes, warns on statuses not in {draft, review, active, stable, deprecated, archived}, and includes specs with no status only when excluding (not when `--only-status` is active)
- `build_schema_columns` returns an empty map when `schema_dir` is unset, otherwise builds column-level schema from migrations
- `run_validation` validates each spec, applies global/inline/per-spec ignore rules to warnings, and returns `(errors, warnings, passed, total, error_strings, warning_strings)`; when `collect` is false it renders the full per-spec text report
- `compute_exit_code` returns: Warn → always 0; EnforceNew → 1 if any unspecced files; Strict → 1 on any error and (with `--strict`) 1 on any warning; and 1 whenever `--require-coverage N` exceeds actual file coverage, regardless of mode
- `exit_with_status` mirrors `compute_exit_code` but prints the reason and calls `process::exit`
- `create_drift_issues` groups `"spec/path: message"` errors by spec and creates exactly one GitHub issue per spec using configured drift labels (default `spec-drift`)

## Constraints

- Must not panic on expected error conditions — print and exit, or return Result-like values
- Must reuse `load_config`, `find_spec_files`, `validate_spec` from the library rather than duplicating logic
- `\r\n` is normalized to `\n` before frontmatter parsing in status filtering
- GitHub issue creation must continue past individual `gh` failures, reporting each per-spec error
- Output rendering must respect the requested `OutputFormat` (text vs collected JSON/markdown/GitHub)

## Out of Scope

- GUI or web interface
- Interactive prompts (only the wizard and the check `--fix` re-validation prompt live elsewhere)
- Domain logic for validation/scoring/coverage — those live in their own library modules; this module only orchestrates them
- Defining the CLI argument grammar (that is the `cli` module's responsibility)

### REQ-commands-001

The system SHALL describe registered command modules using their current persisted layout and behavior.

Acceptance Criteria
- The init registry entry names the `.specsync/` 5.0 layout rather than the removed root JSON layout.
- Command documentation remains consistent with the dispatched modules.

### REQ-commands-002

Check reporting SHALL expose planned mapping notices separately from warnings in text, JSON, Markdown, and GitHub formats.

Acceptance Criteria

- Text output identifies each planned path without printing a misleading all-files-exist check.
- Structured JSON includes a deterministic notices array.
- Markdown and GitHub reports include a planned mappings section.
- Notice-only results remain passing under strict enforcement.
