---
spec: generator.spec.md
---

## Tasks

- [ ] Add a `--force` flag to overwrite existing specs (with a confirmation prompt)
- [ ] Add a `--dry-run` flag to preview what would be generated without writing files
- [ ] Support an interactive mode that lets the user confirm/edit each generated spec before writing
- [ ] Populate companion files (tasks/context/requirements/testing) with AI-authored, source-specific content instead of bare scaffolds

## Done

- [x] Keep module-discovery test fixtures warning-free under current stable Clippy
- [x] Template-based spec generation with language-aware templates (Rust, Swift, Kotlin/Java, Go, Python; default for the rest)
- [x] AI-powered spec generation delegated to `ai::generate_spec_with_ai`, with fallback to template on failure
- [x] Carry the new `corvid-ai`-backed `ResolvedProvider` (passed in as `Option<&ResolvedProvider>`) through generation — no provider-specific logic lives in the generator
- [x] AI mode entered only when a provider is configured; otherwise template-only (no network)
- [x] Custom `_template.spec.md` support and custom template-directory support with per-file fallback
- [x] Companion file generation: tasks.md, context.md, requirements.md, testing.md, and opt-in design.md (with `spec:`/`sources:` frontmatter)
- [x] Module discovery: config `modules` definitions → `src/<module>/` subdirectories → flat `src/<module>.<ext>` files, excluding test files
- [x] Frontmatter rewriting: module name, version 1, status draft, root-relative file list, reset `db_tables`
- [x] Replace the old unfinished-marker template bodies with guided starter content

## Gaps

- Template-only specs still need source-specific expansion (AI or hand-editing) to score well on the quality rubric
- No way to regenerate a single existing spec without deleting it first (`--force` not yet implemented)
- Companion file bodies are scaffolds, not AI-generated

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
