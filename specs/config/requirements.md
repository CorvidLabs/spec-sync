---
spec: config.spec.md
---

## User Stories

- As a developer, I want spec-sync to work with zero configuration by auto-detecting my project structure so that I can try it immediately on any project
- As a developer, I want to use either JSON or TOML for configuration so that I can match my project's conventions
- As a team lead, I want to configure required sections, excluded directories, and source extensions so that validation fits our project's needs
- As a monorepo maintainer, I want spec-sync to discover source directories from manifest files (Cargo.toml, package.json, etc.) so that complex project structures are handled automatically
- As a developer, I want missing config fields to fall back to sensible defaults so that I only need to configure what I want to customize

## Acceptance Criteria

- [ ] Config search order: `specsync.json` > `.specsync.toml` > auto-detected defaults
- [ ] Auto-detection scans up to 3 directory levels deep for source files
- [ ] At least 46 common build/cache directories are excluded from auto-detection (node_modules, target, .git, dist, etc.)
- [ ] Falls back to `["src"]` if no source files are found anywhere
- [ ] Root-level source files produce `["."]` as the source directory
- [ ] TOML parsing works without external TOML dependencies (zero-dependency line-by-line parser)
- [ ] Auto-detection runs even when config file exists but omits `sourceDirs`
- [ ] `load_config` never panics — always returns a valid config with defaults filled in
- [ ] Manifest-aware discovery (Cargo.toml, package.json, etc.) feeds into source directory detection

## Constraints

- Config loading must be fast — no network calls, no AI, no heavy computation
- TOML parser only needs to handle the subset of TOML used by specsync configs (not full TOML spec)
- Config schema must be backwards-compatible — new fields must always have defaults

## Out of Scope

- Config file validation or linting beyond basic parse errors
- Config inheritance or merging across directories (monorepo-level config)
- Remote/shared configuration (config is always local to the project)
- Environment variable overrides for all config fields (only AI-related env vars supported)
