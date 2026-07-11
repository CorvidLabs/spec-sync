## ADDED

### REQUIREMENT REQ-cmd-new-001

The new command SHALL create a non-overwriting spec scaffold from validated module input and detected source exports.

Acceptance Criteria
- `specsync new <module>` creates `<specs_dir>/<module>/<module>.spec.md` with frontmatter (`module`, `version: 1`, `status: draft`, `files:`, `db_tables: []`, `depends_on: []`) and the standard section skeleton (Purpose, Public API, Dependencies, Change Log).
- Source files are detected by scanning each configured `source_dirs` entry for a directory named `<module>` (recursively) and for a top-level file whose stem equals `<module>`, filtered by `source_extensions`; detected paths are relative, forward-slash, sorted, and de-duplicated.
- When no source files are found, the spec is still created with `files: []`.
- Exported symbols from the detected source files are collected via `exports::get_exported_symbols`, de-duplicated, and rendered as Public API rows; each row carries a review prompt to document the export rather than a placeholder marker.
- `--full` invokes `generator::generate_companion_files_for_spec`, creating `tasks.md`, `context.md`, `requirements.md`, `testing.md`, and `design.md` only when `companions.design` is enabled in config.
- An existing target spec file causes exit code 1 with an error message; the command never overwrites it.
