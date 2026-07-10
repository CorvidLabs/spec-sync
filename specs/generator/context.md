---
spec: generator.spec.md
---

## Key Decisions

- **Never overwrite**: existing specs and existing companion files are never overwritten — this protects hand-tuned documentation.
- **Template-only is always available**: AI is opt-in. The generator only enters AI mode when a provider is configured; otherwise it scaffolds from templates and makes no network call.
- **AI delegated to the `ai` module**: the generator passes an `Option<&ResolvedProvider>` straight to `ai::generate_spec_with_ai`. After the 4.4.0 rework, `ResolvedProvider` is `Cli(String) | Api(corvid_ai::Settings)` and all HTTP goes through `corvid-ai` — none of that leaks into the generator, which only orchestrates files and templates.
- **AI falls back to template**: if AI generation fails (missing key, timeout, bad output), `generate_module_spec` catches the error, warns on stderr, and writes the template version so the user always gets a spec.
- **Custom templates first**: a `specs/_template.spec.md`, or a custom template directory passed to `generate_spec_from_custom_template` / `generate_companion_files_from_template`, takes precedence over built-ins, with per-file fallback to the defaults.
- **Language-aware templates**: Rust, Swift, Kotlin/Java, Go, and Python each get tailored Public API sections; TypeScript, C#, Dart, and unknown languages use `DEFAULT_TEMPLATE`. Primary language is the most common source extension.
- **Module discovery order**: config `modules` definitions → `src/<module>/` subdirectories → flat `src/<module>.<ext>` files; test files are excluded via `is_test_file`.
- **Companion frontmatter**: each companion keeps a `spec: <module>.spec.md` back-reference; `design.md` additionally carries `sources: []`. This companion-level metadata is not parsed by the spec validation pipeline.

## Files to Read First

- `src/generator.rs` — template constants, language templates, module discovery (`find_files_for_module`), spec generation, and companion creation all live here.
- `src/ai.rs` — `generate_spec_with_ai` and `ResolvedProvider`; the generator delegates all LLM work here.
- `src/exports/mod.rs` — `has_extension` and `is_test_file`, used to filter discovered source files.
- `src/types.rs` — `SpecSyncConfig` (`modules`, `source_dirs`, `source_extensions`, `companions.design`, `required_sections`) and `CoverageReport`.

## Current Status

Fully implemented and stable. Template-based and AI-powered generation both work; AI generation now flows entirely through the `ai` module's corvid-ai-backed provider. Companion files (tasks/context/requirements/testing, plus opt-in design) are created by `generate_specs_for_unspecced_modules` and exposed via `generate_companion_files_for_spec`. Module-discovery test fixtures pass the current stable Clippy gate without warnings.

## Notes

- Module titles are dash-to-title-case from the module name ("api-gateway" → "Api Gateway").
- Frontmatter rewriting is regex-based: it replaces `module`/`status`/`version`/`files`/`db_tables` and the `# Title` line; the `files:` regex handles both `files: []` and an existing multi-line YAML list.
- `generate_specs_for_unspecced_modules_paths` is the path-returning variant used by the MCP server; the count-returning variant is used by the CLI.
