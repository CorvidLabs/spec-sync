---
spec: commands.spec.md
---

## User Stories

- As a subcommand author, I want shared config-load, discovery, filtering, and validation helpers so that I don't re-implement boilerplate in every `commands::*` module
- As a CLI user, I want `--exclude-status` / `--only-status` and positional spec filters honored consistently across commands so that scoping a run behaves the same everywhere
- As a CI operator, I want exit codes that reflect enforcement mode and coverage thresholds so that pipeline pass/fail is predictable and actionable
- As a maintainer, I want exit-code logic separated from process exit so that the decision is unit-testable without spawning a process
- As a team using GitHub, I want validation errors turned into one issue per drifted spec so that drift gets tracked without flooding the tracker
- As a terminal and GitHub user, I want hostile drift paths and provider text rendered safely so
  that validation data cannot inject terminal or issue formatting
- As a cross-platform user, I want every module name validated against portable component rules so
  that a spec created on Unix cannot fail, alias a device, or exceed component limits on Windows.

## Acceptance Criteria

- `load_and_discover` loads config, discovers spec files under the configured specs dir, excludes `_`-prefixed files, and exits 0 with a "run `specsync generate`" hint when empty and `allow_empty` is false
- `filter_specs` matches a filter against exact path, project-relative path, filename stem, or module name (stem minus `.spec`); empty filters return all specs; unmatched filters print a yellow warning
- `filter_by_status` supports exclude-list and only-list modes, warns on statuses not in {draft, review, active, stable, deprecated, archived}, and includes specs with no status only when excluding (not when `--only-status` is active)
- `build_schema_columns` returns an empty map when `schema_dir` is unset, otherwise builds column-level schema from migrations
- `run_validation` validates each spec, applies global/inline/per-spec ignore rules to warnings, and returns `(errors, warnings, passed, total, error_strings, warning_strings)`; when `collect` is false it renders the full per-spec text report
- `compute_exit_code` returns: Warn → always 0; EnforceNew → 1 if any unspecced files; Strict → 1 on any error and (with `--strict`) 1 on any warning; and 1 whenever `--require-coverage N` exceeds actual file coverage, regardless of mode
- `exit_with_status` mirrors `compute_exit_code` but prints the reason and calls `process::exit`
- `create_drift_issues` preserves its public rendered-string input contract, attributes errors
  against the longest exact discovered spec-path prefix, and creates exactly one GitHub issue per
  spec using configured drift labels (default `spec-drift`), including legal paths containing
  `": "`.
- `create_drift_issues` sanitizes untrusted repository-resolution failures, spec paths, created
  issue URLs, and provider failures before terminal rendering; delegated GitHub issue titles and
  bodies are sanitized independently
- `validate_module_name` rejects path traversal/control/Windows-invalid characters, trailing
  spaces/dots, case-insensitive Windows device basenames (including before an extension), and names
  over 247 UTF-8 bytes; 247-byte ASCII and multibyte boundaries remain valid.

## Constraints

- Must not panic on expected error conditions — print and exit, or return Result-like values
- Must reuse `load_config`, `find_spec_files`, `validate_spec` from the library rather than duplicating logic
- `\r\n` is normalized to `\n` before frontmatter parsing in status filtering
- GitHub issue creation must continue past individual `gh` failures, reporting each per-spec error
- Drift-creation renderers must not emit raw terminal controls, bidirectional formatting controls,
  or Unicode line/paragraph separators from repository, path, URL, or provider text
- Output rendering must respect the requested `OutputFormat` (text vs collected JSON/markdown/GitHub)
- Generated `<module>.spec.md` filenames must fit within one portable 255-byte component.

## Out of Scope

- GUI or web interface
- Interactive prompts (only the wizard and the check `--fix` re-validation prompt live elsewhere)
- Domain logic for validation/scoring/coverage — those live in their own library modules; this module only orchestrates them
- Defining the CLI argument grammar (that is the `cli` module's responsibility)

### REQ-commands-001

The `commands` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-commands-002

Check reporting SHALL expose planned mapping notices separately from warnings in text, JSON, Markdown, and GitHub formats.

Acceptance Criteria

- `run_validation` returns deterministic notice strings as a seventh tuple member.
- Text output identifies each planned path without printing a misleading all-files-exist check.
- Structured JSON includes a deterministic notices array on normal, SDD-error, unmatched-filter, and no-spec exit paths.
- Markdown and GitHub reports include a planned mappings section.
- Notice-only results remain passing under strict enforcement.

### REQ-commands-003

Drift-issue creation SHALL render untrusted text safely at both the command terminal and GitHub
issue boundaries.

Acceptance Criteria

- Repository-resolution failures, spec paths, returned issue URLs, and provider failures pass
  through the shared safe diagnostic renderer before terminal output.
- Terminal output does not preserve raw control characters, bidirectional formatting controls, or
  Unicode line/paragraph separators from untrusted values.
- The explicit GitHub creation helper sanitizes spec paths and validation errors separately for
  title text and Markdown body text.
- Sanitization does not change grouping: one drift issue is still attempted per spec, and an
  individual creation failure does not stop later specs.
- Public validation retains its rendered `Vec<String>` diagnostics contract. Private structured
  attribution and longest exact discovered-path matching preserve legal paths containing `": "`
  without exporting new command types.

### REQ-commands-004

Shared module-name validation SHALL enforce one portable generated-spec component contract on every
host.

Acceptance Criteria

- Reject Windows reserved device basenames case-insensitively, including before extensions.
- Reject Windows-invalid component characters on every host.
- Reject trailing spaces/dots and names longer than 247 UTF-8 bytes so `<name>.spec.md` remains at
  most 255 bytes.
- Preserve valid ASCII and multibyte names exactly at the 247-byte boundary.

### REQ-commands-005

Command orchestration SHALL preserve fallible schema validation and visible ignore suppression
without reporting false success.

Acceptance Criteria

- A schema snapshot failure is returned as an error and cannot become an empty successful comparison.
- Text and structured check/report outputs distinguish emitted warnings from suppressed warnings.
- Suppression details are deterministic across text, JSON, Markdown, and GitHub formats.
- Existing notice, strict, coverage, and exit semantics remain compatible except where a prior path
  falsely reported success.

### REQ-commands-change-audit-dispatch-001

The change command dispatcher SHALL route `Audit` to active-only project audit and `Check` to scoped verification without dual-wiring full archive integrity into check.

Acceptance Criteria
- Check path does not call full archive integrity.
- Audit path fails closed on active/living-spec errors only.

### REQ-commands-006

A spec with `status: draft` SHALL be reported when it skips validation it could have
performed, and SHALL NOT be reported otherwise.

Acceptance Criteria
- A draft spec produces a warning when at least one mapped source file was present and readable AND its Public API names at least one symbol.
- A draft spec whose mapped files do not exist yet produces no such warning and continues to pass strict validation.
- A draft spec whose Public API names no symbol produces no such warning.
- Bare `specsync check` remains exit 0 in every draft case; only strict mode gates.

### REQ-commands-007

Strict validation SHALL refuse to report success for a tree whose coverage excluded skipped symlinked entries.

Acceptance Criteria
- Strict mode exits non-zero when any entry was skipped, naming how many.
- Bare validation continues to exit zero and only reports the exclusion.
- Both the text and machine-readable exit paths apply the same rule.

### REQ-commands-008

Validation output SHALL NOT report a successful result for a check that could not run.

Acceptance Criteria
- When frontmatter is invalid, the source-file, DB-table, required-section, and dependency checks are each reported as skipped rather than as passing.
- The skipped form matches the existing vocabulary used when a draft spec skips validation.
- A spec with valid frontmatter continues to report all four checks unchanged, including a declared table absent from the schema and every genuinely missing required section.
- The exit status is unaffected: invalid frontmatter remains an error.

### REQ-commands-009

No command SHALL report a verdict derived from configuration that failed to load.

Acceptance Criteria
- A command that reads specs refuses to run when the configuration records a load failure, and names the file.
- The refusal states how to proceed: fix the file, or remove it to use the built-in defaults deliberately.
- A project with a valid configuration, and a project with none, are both unaffected.
- The refusal is applied once at the shared entry point, so no command can omit it.
