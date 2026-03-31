---
spec: types.spec.md
---

## User Stories

- As a spec-sync contributor, I want all shared types defined in one module so that other modules import from a single source of truth
- As a developer adding a new AI provider, I want the AiProvider enum to be extensible with clear methods (default_command, binary_name, api_key_env_var) so that adding a provider is straightforward
- As a developer adding a new language, I want the Language enum to map extensions to languages and provide test patterns so that language support is consistent
- As an integrator consuming spec-sync output, I want ValidationResult and CoverageReport to be well-structured so that I can process results programmatically

## Acceptance Criteria

- [ ] AiProvider enum includes: Claude, Cursor, Copilot, Ollama, Anthropic, OpenAi, Custom
- [ ] `AiProvider::from_str_loose` is case-insensitive and supports common aliases
- [ ] `AiProvider::detection_order` returns CLI providers before API providers
- [ ] Language enum covers all 11 supported languages with correct extension mappings
- [ ] `Language::from_extension` returns None for unsupported extensions (no panic)
- [ ] `Language::test_patterns` returns language-appropriate test file patterns
- [ ] OutputFormat enum includes Text, Json, and Markdown variants
- [ ] ExportLevel enum has Type (class/struct/enum only) and Member (all public symbols, default) variants
- [ ] `SpecSyncConfig::default()` provides sensible defaults for all fields
- [ ] `ValidationResult::new()` initializes with empty error/warning/fix vectors
- [ ] All types derive necessary serde traits for JSON serialization where needed

## Constraints

- Types module must have zero dependencies on other spec-sync modules (it's the foundation)
- All enums must be exhaustive — no catch-all variants that hide missing match arms
- Default implementations must produce valid, usable values

## Out of Scope

- Runtime type validation (types are validated at compile time via Rust's type system)
- Serialization format negotiation (JSON is the only serialized format)
- Backwards-compatible type versioning
