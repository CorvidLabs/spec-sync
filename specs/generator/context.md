---
spec: generator.spec.md
---

## Key Decisions

- **Never overwrite**: Existing specs are never overwritten by generation — this prevents accidental loss of hand-tuned documentation.
- **Custom templates first**: If `_template.spec.md` exists in the specs directory, it's used instead of the built-in default. This lets projects define their own spec structure.
- **Language-specific templates**: Built-in templates vary by language (Rust, Swift, Kotlin/Java, Go, Python each have tailored Public API table formats). Generic template covers TypeScript, C#, Dart.
- **AI fallback to template**: If AI generation fails (provider unavailable, timeout, bad output), the template-based generator runs as a fallback so the user always gets something.
- **Companion file generation**: `tasks.md` and `context.md` are generated alongside specs as empty scaffolds. Only created if they don't already exist.
- **Flat file detection**: Standalone source files (not in subdirectories) are detected as modules, excluding common entry points like main, lib, mod, index, app, `__init__`.

## Files to Read First

- `src/generator.rs` — Single-file module handling spec generation, template selection, and companion file creation.

## Current Status

Fully implemented. Template-based and AI-powered generation both work. Companion files are generated as part of the `generate` command and also via `generate_companion_files_for_spec()`.

## Notes

- Module titles are derived from directory/file names with dash-to-title-case conversion (e.g., "api-gateway" becomes "Api Gateway").
- The generator consumes the exports module to pre-populate Public API tables in templates.
