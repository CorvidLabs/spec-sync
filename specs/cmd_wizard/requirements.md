---
spec: cmd_wizard.spec.md
---

## User Stories

- As a developer new to a project, I want an interactive wizard that walks me through creating a spec (name, purpose, type, status, sources, dependencies) so I don't have to remember the frontmatter format
- As a developer, I want the wizard to auto-detect source files for my module and let me add one manually when nothing is found
- As a developer, I want a preview and an explicit confirmation before anything is written so I can cancel a mistake
- As a developer, I want module-type presets (API endpoint, data model, utility, UI component) that seed type-appropriate invariants and API sections

## Acceptance Criteria

- Prompts (via `dialoguer`) collect, in order: module name, purpose, module type, initial status, source files, dependencies
- Module name is trimmed; an empty name prints an error and exits 1
- If `<specs_dir>/<module>/<module>.spec.md` already exists, the wizard prints a warning and exits 1 (never overwrites)
- Source files are auto-detected by scanning `config.source_dirs` for files whose stem or parent directory equals the module name and whose extension is in `source_extensions`; when none are found the user may enter one path or skip
- Status choices are `draft`, `unstable`, `stable`, `locked` (default `draft`); module-type choices add type-specific invariants/API hints
- Dependencies are parsed from a comma-separated list into `depends_on`
- A truncated preview (~first 30 lines) is shown, then a write confirmation; declining prints "Cancelled." and returns without writing
- On confirm, the spec dir is created, the spec is written, and companion files are generated (design.md only when `config.companions.design` is enabled)
- Cancelling at any prompt (Ctrl-C / interrupt) exits cleanly with code 0

## Constraints

- Must not panic on expected error conditions — print and exit
- Interactive only: relies on a TTY for `Input`/`Select`/`Confirm` prompts
- Read-only with respect to existing specs: an existing `*.spec.md` is never modified
- Source-file paths are normalized to forward slashes in generated frontmatter

## Out of Scope

- Non-interactive / scripted spec creation (use `scaffold` or `add-spec`)
- Coding-agent enrichment after the wizard writes its deterministic scaffold
- Editing or re-running against an existing spec

### REQ-cmd-wizard-001

The `cmd_wizard` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.


### REQ-cmd-wizard-002

`wizard` SHALL keep working over a broken configuration.

Acceptance Criteria
- It requests the permissive loader explicitly, because a command whose job is repairing the config must run on the project that needs repairing.

### REQ-cmd-wizard-003

The wizard SHALL create the spec file only through `create_project_file` of `util` (REQ-util-002) and its companions through the generator (REQ-generator-010), so a spec path that is a dangling link, or leaves the project through a link, is refused with an error naming the path and nothing is created outside the project.

Acceptance Criteria
- The wizard's write step, driven with a dangling link at the spec path, returns an error naming the path and creates nothing at the link's target.
- The same step with the spec directory linked to a directory outside the project creates nothing there.

### REQ-cmd-wizard-004

The wizard SHALL find candidate source files only through the confined, budgeted walk of `util`, so a source folder that leads out of the project is never walked.

Acceptance Criteria
- The wizard's source walk uses `find_project_files`.

