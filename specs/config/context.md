---
spec: config.spec.md
---

## Key Decisions

- **v4 layout precedence**: `load_config` checks `.specsync/config.toml` first, then `.specsync/config.json`, then legacy root `.specsync.toml`, then legacy root `specsync.json`, then auto-detected defaults. The TOML `.specsync/config.toml` is the canonical v4 form; the JSON variants stay supported for back-compat.
- **Per-developer local overrides**: After the shared config loads, `.specsync/config.local.toml` (gitignored) is merged on top via `merge_local_config`. Only top-level `ai_*` keys are applied — any key inside a `[section]` is skipped, and unknown top-level keys warn "only ai_* keys are supported." This lets contributors set their own `ai_provider`/`ai_command`/`ai_model` without touching committed config.
- **Zero-dependency TOML**: Rather than pulling in a TOML crate, parsing is done line-by-line with string operations, routed per section (`[rules]`, `[github]`, `[lifecycle]`, `[lifecycle.max_age]`, `[lifecycle.guards."x→y"]`, `[companions]`). This keeps the dependency tree minimal and avoids version conflicts.
- **AI fields are loose-parsed**: `ai_provider`/`aiProvider` goes through `AiProvider::from_str_loose`, so aliases like `claude` (deprecated → anthropic) and casing variants resolve. The env-precedence layering (`SPECSYNC_AI_PROVIDER`, `SPECSYNC_AI_COMMAND`, `SPECSYNC_AI_MODEL`, `<PROVIDER>_API_KEY`) is handled by the `ai`/`cmd_generate` layers, not here — `config.rs` only reads file values.
- **API key never written back**: `config_to_toml` intentionally omits `ai_api_key` and prints a warning telling the user to set the `<PROVIDER>_API_KEY` env var instead, so secrets don't land in committed config.
- **Auto-detection fallback**: If no config file exists (or a file omits source dirs), source directories are detected by scanning for files with recognized extensions up to 3 levels deep. Fallback is `["src"]` if nothing found; root-level source files yield `["."]`.
- **Manifest-first discovery**: When detecting source dirs, manifest files (Cargo.toml, Package.swift, etc.) are checked before falling back to extension scanning. This gives more accurate module-aware results.
- **46 hardcoded excludes**: Common build/cache directories (node_modules, target, .git, dist, etc.) are always excluded to prevent scanning generated code.

## Files to Read First

- `src/config.rs` — Config loading, format precedence, `merge_local_config`, TOML/JSON parsing, `config_to_toml`, `is_legacy_layout`, source directory detection, and manifest integration.
- `src/types.rs` — `SpecSyncConfig`, `AiProvider`, and the per-section config structs (rules, github, lifecycle, companions) with all field defaults.
- `src/ai.rs` — Where AI env precedence and provider resolution actually happen (config supplies the file-level values they layer on top of).

## Current Status

Fully implemented for spec-sync 4.4.0. JSON and TOML loading work with unknown-key warnings, v4 layout precedence, and the `config.local.toml` AI override merge. AI config is aligned with the reworked corvid-ai providers (anthropic, openai, openrouter, gemini, deepseek, groq, mistral, xai, together, ollama; deprecated claude/copilot/cursor; plus `ai_command` shell hatch). Auto-detection covers the supported languages and delegates manifest discovery to the manifest module. Round-trip test fixtures pass the current stable Clippy gate without warnings.

## Notes

- The config module is the bridge between user intent (config file) and the rest of the system — validator, watch, MCP, and the generate command all depend on it.
- Unknown keys in config files produce warnings, not errors, for forward compatibility when newer config options are added.
- `config.rs` does not read env vars itself (except indirectly via callers); the AI env-var precedence ladder lives in `src/ai.rs` and `src/commands/generate.rs`.
