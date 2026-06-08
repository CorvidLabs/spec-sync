---
spec: config.spec.md
---

## Tasks

- [ ] Support config file extends/inheritance (`"extends": "./base-specsync.json"`)
- [ ] Add config validation with actionable error messages for invalid field values
- [ ] Support environment variable interpolation in config paths (e.g., `$HOME/specs`)

## Done

- [x] JSON config loading with field defaults (camelCase keys, unknown-key warnings)
- [x] TOML config loading (zero-dependency parser) incl. `[rules]`, `[github]`, `[lifecycle]`, `[lifecycle.max_age]`, `[lifecycle.guards."x→y"]`, `[companions]` sections
- [x] v4 layout config precedence: `.specsync/config.toml` > `.specsync/config.json` > `.specsync.toml` > `specsync.json`
- [x] `.specsync/config.local.toml` per-developer override merge (top-level `ai_*` keys only)
- [x] AI config fields wired through both formats: `aiProvider`/`ai_provider` (loose-parsed), `aiModel`, `aiCommand`, `aiApiKey`, `aiBaseUrl`, `aiTimeout`
- [x] `config_to_toml` serializer — round-trips config, omits `ai_api_key` with a "set the env var instead" warning
- [x] `is_legacy_layout` detection for 3.x root-level layouts
- [x] Inline-comment stripping in local config that preserves `#` inside quoted strings (`strip_inline_comment`)
- [x] Auto-detection of source directories by file extension
- [x] Manifest-aware source directory discovery
- [x] 46 hardcoded build/cache directory exclusions
- [x] Unknown key warnings for forward compatibility (`KNOWN_JSON_KEYS`, per-section TOML warnings)

## Gaps

- TOML parsing handles flat key-value pairs, simple inline arrays, and known sections only — arbitrary nested tables outside the known sections are skipped silently
- No schema validation beyond type checking (e.g., invalid `status` values in modules aren't caught at config load time)
- Non-AI fields cannot be overridden in `config.local.toml` (intentional, but warns rather than supporting it)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
