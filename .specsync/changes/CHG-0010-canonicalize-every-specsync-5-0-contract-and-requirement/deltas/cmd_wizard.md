## ADDED

### REQUIREMENT REQ-cmd-wizard-001

The wizard command SHALL guide validated spec creation interactively while preserving the same no-overwrite contract as non-interactive scaffolding.

Acceptance Criteria
- Prompts (via `dialoguer`) collect, in order: module name, purpose, module type, initial status, source files, dependencies
- Module name is trimmed; an empty name prints an error and exits 1
- If `<specs_dir>/<module>/<module>.spec.md` already exists, the wizard prints a warning and exits 1 (never overwrites)
- Source files are auto-detected by scanning `config.source_dirs` for files whose stem or parent directory equals the module name and whose extension is in `source_extensions`; when none are found the user may enter one path or skip
- Status choices are `draft`, `unstable`, `stable`, `locked` (default `draft`); module-type choices add type-specific invariants/API hints
- Dependencies are parsed from a comma-separated list into `depends_on`
- A truncated preview (~first 30 lines) is shown, then a write confirmation; declining prints "Cancelled." and returns without writing
- On confirm, the spec dir is created, the spec is written, and companion files are generated (design.md only when `config.companions.design` is enabled)
- Cancelling at any prompt (Ctrl-C / interrupt) exits cleanly with code 0
