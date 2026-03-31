---
spec: generator.spec.md
---

## User Stories

- As a developer adopting spec-sync on an existing project, I want to generate spec scaffolds for all unspecced modules in one command so that I have a starting point for every module
- As a developer, I want AI-powered generation to produce specs with real content (not just templates) so that I spend less time writing from scratch
- As a developer, I want companion files (tasks.md, context.md, requirements.md) generated alongside each spec so that the full documentation structure is ready immediately
- As a team, I want a custom `_template.spec.md` to be used when it exists so that generated specs match our conventions
- As a developer, I want existing specs and companion files to never be overwritten so that my manual edits are safe

## Acceptance Criteria

- Specs are never overwritten — modules with existing specs are skipped
- Companion files (tasks.md, context.md, requirements.md) are only created if they don't already exist
- Custom template at `specs/_template.spec.md` takes precedence over the built-in default
- Template fills in module name, version (1), status (draft), and discovered source files
- Module title is derived from directory name with title case ("api-gateway" -> "Api Gateway")
- AI generation falls back to template on failure, with a warning to stderr
- Source file paths in generated specs are relative to the project root
- `generate_specs_for_unspecced_modules` returns the count of specs created
- `generate_specs_for_unspecced_modules_paths` returns the file paths of specs created

## Constraints

- Generator must work without AI (template-only mode is always available)
- Must not make network calls unless AI generation is explicitly requested
- Generated specs must pass `specsync check` without errors (valid frontmatter, required sections present)

## Out of Scope

- Regenerating or updating existing specs based on code changes
- Generating specs for individual files (only module-level)
- Interactive prompts asking the developer to fill in sections
- Generating specs from external documentation or API schemas
