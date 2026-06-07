---
spec: generator.spec.md
---

## User Stories

- As a developer adopting spec-sync on an existing project, I want to scaffold spec files for all unspecced modules in one command so that every module has a starting point
- As a developer with an AI provider configured, I want AI-powered generation to produce real spec content (not just a template) so that I write less from scratch
- As a developer with nothing configured, I want template-only generation so that I never depend on an AI provider or network call to bootstrap specs
- As a developer, I want companion files (tasks.md, context.md, requirements.md, testing.md, and design.md when enabled) generated alongside each spec so that the full documentation structure is ready immediately
- As a team, I want a custom `_template.spec.md` (or a custom template directory) honored when present so that generated specs match our conventions
- As a developer, I want existing specs and companion files to never be overwritten so that my manual edits are safe

## Acceptance Criteria

- Specs are never overwritten — a module with an existing `<module>.spec.md` is skipped
- Companion files are only created when absent; existing companions are left untouched
- `generate` enters AI mode only when a provider is configured (`--provider`, `aiProvider`/`aiCommand`, or `SPECSYNC_AI_PROVIDER`/`SPECSYNC_AI_COMMAND`); with nothing configured it is template-only and makes no network calls
- AI generation falls back to template generation on any failure, printing a warning to stderr (the spec is still written)
- A custom `specs/_template.spec.md` takes precedence over the built-in language-aware default; `generate_spec_from_custom_template` / `generate_companion_files_from_template` honor a custom template directory and fall back to built-ins per-file
- Built-in templates are language-aware: Rust, Swift, Kotlin/Java, Go, and Python each get tailored Public API sections; TypeScript, C#, Dart, and unknown fall back to the default template
- Templates fill in module name, version (1), status (draft), and discovered source-file paths (relative to root); `db_tables` is reset to `[]`
- Module title is dash-to-title-case from the module name ("api-gateway" → "Api Gateway")
- Test files are excluded from module source discovery (via `is_test_file`)
- Module discovery order: config `modules` definitions first, then `src/<module>/` subdirectories, then flat `src/<module>.<ext>` files
- `generate_specs_for_unspecced_modules` returns the count of specs created; `generate_specs_for_unspecced_modules_paths` returns their paths
- `design.md` is generated only when `companions.design` is enabled, and carries its own frontmatter (`spec:` back-reference, `sources: []`)

## Constraints

- Template-only mode must always be available; the generator must work with no AI provider
- No network calls unless an AI provider is explicitly configured for the run
- AI prompt building, truncation, and the LLM call are delegated to the `ai` module (`generate_spec_with_ai`); the generator owns only template scaffolding and file orchestration
- Generated specs must pass `specsync check` (valid frontmatter, required sections present)
- Source-file paths written into frontmatter are relative to the project root

## Out of Scope

- Regenerating or updating existing specs from code drift (handled via the `ai` module's `regenerate_spec_with_ai`, driven elsewhere)
- Per-file specs (generation is module-level only)
- Interactive prompts asking the developer to complete sections
- A `--force` overwrite or `--dry-run` preview (not yet implemented)
- AI-authored companion file bodies (companions are scaffolds; only the `.spec.md` is AI-generated)
