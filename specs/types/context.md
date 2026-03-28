---
spec: types.spec.md
---

## Key Decisions

- **Central type definitions**: All shared types live in `types.rs` rather than being scattered across modules. This creates a single source of truth and prevents circular dependencies.
- **Loose string parsing for AI providers**: `AiProvider::from_str_loose()` accepts case-insensitive input with common aliases (e.g., "gh-copilot" → Copilot, "gpt" → OpenAI). This makes CLI input forgiving without sacrificing type safety internally.
- **CLI providers before API providers**: The auto-detection order checks for installed CLI tools (Claude, Ollama, Copilot) before checking for API keys (Anthropic, OpenAI), preferring the simpler integration path.
- **Sensible defaults everywhere**: `SpecSyncConfig::default()` provides working values for all fields (specs_dir="specs", source_dirs=["src"], required sections, etc.) so the tool works without any config file.
- **Never panics**: All defaults are always provided, and the type system ensures no field access can cause a panic from missing data.

## Files to Read First

- `src/types.rs` — Single-file module defining all enums, structs, and their `Default`/`Display`/`FromStr` implementations.

## Current Status

Fully implemented. The types module is stable and consumed by every other module in the project. Changes here have the widest blast radius.

## Notes

- `ModuleDefinition` in config allows users to explicitly define modules with their source files, overriding auto-detection. This is the escape hatch for projects with non-standard layouts.
- The `OutputFormat` enum (Text, Json, Markdown) determines CLI output formatting across all reporting commands.
