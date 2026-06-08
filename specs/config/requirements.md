---
spec: config.spec.md
---

## User Stories

- As a developer, I want spec-sync to work with zero configuration by auto-detecting my project structure so that I can try it immediately on any project
- As a developer, I want to use either JSON or TOML for configuration so that I can match my project's conventions
- As a team lead, I want to configure required sections, excluded directories, and source extensions so that validation fits our project's needs
- As a monorepo maintainer, I want spec-sync to discover source directories from manifest files (Cargo.toml, package.json, etc.) so that complex project structures are handled automatically
- As a developer, I want missing config fields to fall back to sensible defaults so that I only need to configure what I want to customize
- As a contributor on a shared repo, I want per-developer AI settings in a gitignored `.specsync/config.local.toml` so that my chosen AI provider/model doesn't conflict with the team's committed config
- As a developer, I want to pick an AI provider via the flat `aiProvider`/`aiModel`/`aiCommand`/`aiApiKey`/`aiBaseUrl`/`aiTimeout` fields so that `generate` can run AI without re-passing flags

## Acceptance Criteria

- Config search order: `.specsync/config.toml` > `.specsync/config.json` > `.specsync.toml` (legacy root) > `specsync.json` (legacy root) > auto-detected defaults
- After loading the shared config, `.specsync/config.local.toml` is merged on top when present; only top-level `ai_*` keys are honored there (section keys are skipped)
- Auto-detection scans up to 3 directory levels deep for source files
- 46 common build/cache directories are excluded from auto-detection (node_modules, target, .git, dist, etc.)
- Falls back to `["src"]` if no source files are found anywhere
- Root-level source files produce `["."]` as the source directory
- TOML parsing works without external TOML dependencies (zero-dependency line-by-line parser)
- Auto-detection runs even when config file exists but omits `sourceDirs`/`source_dirs`
- `load_config` never panics — always returns a valid config with defaults filled in
- Manifest-aware discovery (Cargo.toml, package.json, etc.) feeds into source directory detection
- AI fields are parsed from both JSON (`aiProvider`, `aiModel`, `aiCommand`, `aiApiKey`, `aiBaseUrl`, `aiTimeout`) and TOML (`ai_provider`, `ai_model`, `ai_command`, `ai_api_key`, `ai_base_url`, `ai_timeout`); `ai_provider`/`aiProvider` is parsed loosely via `AiProvider::from_str_loose` (e.g. `claude`, `anthropic`, `ollama`)
- `config_to_toml` round-trips config back to TOML but deliberately omits `ai_api_key`, printing a warning that the value should live in a `<PROVIDER>_API_KEY` env var instead
- Inline TOML comments are stripped from `config.local.toml` values, but a `#` inside a quoted string is preserved
- `is_legacy_layout` reports true when root-level config (`specsync.json`/`.specsync.toml`/`specsync-registry.toml`) exists without a `.specsync/version` stamp

## Constraints

- Config loading must be fast — no network calls, no AI, no heavy computation
- TOML parser only needs to handle the subset of TOML used by specsync configs (not full TOML spec)
- Config schema must be backwards-compatible — new fields must always have defaults
- `ai_api_key` must never be written back into a committed config file by `config_to_toml`
- `config.local.toml` overrides are limited to AI fields by design — other keys emit an "only ai_* keys are supported" warning

## Out of Scope

- Config file validation or linting beyond basic parse errors and unknown-key warnings
- Config inheritance/`extends` across files (only the fixed local-override merge is supported)
- Remote/shared configuration (config is always local to the project)
- Environment variable overrides for non-AI config fields (env precedence for `aiProvider`/`aiModel`/`aiCommand` is resolved in the `ai`/`cmd_generate` layers, not in `config.rs`)
- Secret management for `ai_api_key` beyond the "use an env var instead" warning
